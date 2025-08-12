use eyre::Result;
use futures_util::{future, SinkExt, StreamExt, TryStreamExt};
use tokio::{net::TcpListener, spawn};
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;

    while let Ok((socket, addr)) = listener.accept().await {
        println!("Accepting {}", addr);

        spawn(async move {
            let websocket = accept_async(socket).await.unwrap();
            let (mut write, read) = websocket.split();

            let mut rx = read.try_filter(|msg| future::ready(msg.is_binary() || msg.is_text()));
            while rx.next().await.is_some_and(|r| r.is_ok()) {
                write
                    .send(Message::Text("Hello, World!".into()))
                    .await
                    .unwrap();
            }

            println!("{} disconnected", addr);
        });
    }

    return Ok(());
}
