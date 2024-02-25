mod event;
mod message;
mod req;

use crate::{
    event::{EventKind, UnsignedEvent},
    message::ClientMessage,
    req::{Filter, Req},
};
use dotenvy;
use futures_util::{SinkExt, StreamExt};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    // dotenv 読み込み
    dotenvy::from_filename(".env").unwrap();

    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();

    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    let pubkey = "be54d42e1c629a90d6644967f4cb8d86ef14b837a7ae8bc97f0ab3eded25d534".to_string();
    let seckey = std::env::var("SECKEY").unwrap();

    let req = Req {
        id: "testtesttesttesttest".to_string(),
        filter: Filter::new()
            .kinds(vec![1])
            .authors(vec![pubkey.to_string()]),
    };

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let event = UnsignedEvent::new(
        pubkey.to_string(),
        EventKind::TextNote,
        Vec::new(),
        "testtesttest".to_string(),
        created_at,
    );
    let event = event.sign(&seckey);
    write
        .send(
            serde_json::to_string(&ClientMessage::from(req))
                .unwrap()
                .into(),
        )
        .await
        .unwrap();
    write
        .send(
            serde_json::to_string(&ClientMessage::from(event))
                .unwrap()
                .into(),
        )
        .await
        .unwrap();
    ws_to_stdout.await;
}
