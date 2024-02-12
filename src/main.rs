mod req;

use crate::req::{Filter, Req};
use futures_util::{pin_mut, SinkExt, StreamExt};
use std::env;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
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

    let req = Req {
        id: "testtesttesttesttest".to_string(),
        filter: Filter::new().kinds(vec![1]),
    };
    write.send(req.serialize().into()).await.unwrap();

    ws_to_stdout.await;
}
