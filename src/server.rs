use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::{ops::ControlFlow, sync::Arc};
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use axum::extract::connect_info::ConnectInfo;

use futures::{stream::StreamExt, SinkExt};

use crate::{
    error::NostrError,
    event::Event,
    message::{ClientMessage, ServerMessage, ServerOk},
    req::{Filter, Req},
    subscriber::Subscriber,
};

#[derive(Clone)]
struct RelayState {
    // サブスクライバーのリスト
    // 接続毎に複数のサブスクライバーを登録可能
    // HashMapのkeyはクライアントのアドレス
    subscribers: Arc<RwLock<HashMap<String, Vec<Subscriber>>>>,
}

pub async fn serve() {
    let state = RelayState {
        subscribers: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<RelayState>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, state, addr))
}

async fn handle_socket(socket: WebSocket, state: RelayState, who: SocketAddr) {
    let (mut sock_tx, mut sock_rx) = socket.split();
    // socketのsenderにメッセージを送信するためのチャンネル
    // socketのsenderを使って複数箇所から送信を行うのが難しいのでチャネルを経由させる
    let (message_tx, mut message_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    if let Some(msg) = sock_rx.next().await {
        if let Ok(msg) = msg {
            if process_message(msg, state.clone(), who, message_tx.clone())
                .await
                .is_break()
            {
                return;
            }
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }

    // メッセージ送信用タスクを開始
    let _ = tokio::spawn(async move {
        while let Some(msg) = message_rx.recv().await {
            let _ = sock_tx.send(msg).await;
        }
    });

    // メッセージ受信用タスクを開始
    let _ = tokio::spawn(async move {
        while let Some(Ok(msg)) = sock_rx.next().await {
            // print message and break if instructed to do so
            if process_message(msg, state.clone(), who, message_tx.clone())
                .await
                .is_break()
            {
                break;
            }
        }
        state.subscribers.write().await.remove(&who.to_string());
    });
}

async fn process_message(
    msg: Message,
    state: RelayState,
    who: SocketAddr,
    message_sender: UnboundedSender<Message>,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} sent str: {t:?}");
            match process_nostr_message(t, state, who, message_sender).await {
                Ok(_) => return ControlFlow::Continue(()),
                Err(e) => {
                    println!(">>> {who} sent invalid message: {e}");
                    tracing::error!("{}", e.to_string());
                    return ControlFlow::Continue(());
                }
            };
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        _ => ControlFlow::Continue(()),
    }
}

// メッセージの処理
async fn process_nostr_message(
    message: String,
    state: RelayState,
    who: SocketAddr,
    message_sender: UnboundedSender<Message>,
) -> Result<(), NostrError> {
    let message: ClientMessage =
        serde_json::from_str(&message).map_err(|e| NostrError::InvalidMessage(e.to_string()))?;

    match message {
        ClientMessage::Req(req) => process_req_message(req, state, who, message_sender).await,
        ClientMessage::Event(event) => process_event_message(event, state, message_sender).await,
        ClientMessage::Close(id) => process_close_message(id, state, who).await,
    }
}

async fn process_req_message(
    req: Req,
    state: RelayState,
    who: SocketAddr,
    message_sender: UnboundedSender<Message>,
) -> Result<(), NostrError> {
    // サブスクリプション登録
    state
        .subscribers
        .write()
        .await
        .entry(who.to_string())
        .or_insert(Vec::new())
        .push(Subscriber {
            client: who.to_string(),
            sender: message_sender,
            id: req.id,
            filter: req.filter,
        });

    Ok(())
}

async fn process_event_message(
    event: Event,
    state: RelayState,
    message_sender: UnboundedSender<Message>,
) -> Result<(), NostrError> {
    // OKメッセージを送信
    let _ = message_sender.send(Message::Text(
        serde_json::to_string(&ServerMessage::Ok(ServerOk {
            event_id: event.id.clone(),
            accepted: true,
            message: "".to_string(),
        }))
        .unwrap(),
    ));

    for s in state
        .subscribers
        .read()
        .await
        .iter()
        .flat_map(|(_, subscribers)| subscribers)
    {
        // サブスクライバーにイベントを送信
        // ここで、イベントがフィルタに合致するかどうかをチェックする
        if match_event(&event, &s.filter) {
            let _ = message_sender.send(Message::Text(serde_json::to_string(&event).unwrap()));
        }
    }
    Ok(())
}

fn match_event(event: &Event, filter: &Filter) -> bool {
    contains(filter.ids.as_ref(), &event.id)
        && (contains(filter.authors.as_ref(), &event.pubkey))
        && (contains(filter.kinds.as_ref(), &u16::from(event.kind)))
        && (filter.e_tags.iter().any(|e| {
            e.iter()
                .any(|tag| event.tags.iter().any(|t| t.contains(tag)))
        }))
        && (filter.p_tags.iter().any(|p| {
            p.iter()
                .any(|tag| event.tags.iter().any(|t| t.contains(tag)))
        }))
        && (filter.since.is_none() || filter.since.unwrap() < event.created_at)
        && (filter.until.is_none() || filter.until.unwrap() > event.created_at)
}

fn contains<T>(vec: Option<&Vec<T>>, item: &T) -> bool
where
    T: PartialEq,
{
    // フィルタが指定されていない場合は、常にtrueを返す
    vec.map_or(true, |v| v.contains(item))
}

async fn process_close_message(
    id: String,
    state: RelayState,
    who: SocketAddr,
) -> Result<(), NostrError> {
    // サブスクリプション登録解除
    let mut subscribers = state.subscribers.write().await;
    if let Some(subscribers) = subscribers.get_mut(&who.to_string()) {
        subscribers.retain(|s| s.id != id);
    }
    Ok(())
}
