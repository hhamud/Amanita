use amanita_common::{transfer_new_file, walk_dir, Data, Directory};
use env_logger::Builder;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use std::{borrow::Cow, path::PathBuf};
use url::Url;

use serde_json;

use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};

type SenderStream = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type RecieverStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Debug)]
pub struct Sender {
    from: PathBuf,
    to: Url,
}

impl Sender {
    fn new(from: PathBuf, to: Url) -> Self {
        Builder::new()
            .filter_level(log::LevelFilter::Info)
            .parse_env("LOG")
            .init();

        Self { from, to }
    }

    async fn connect(&self) -> Result<(SenderStream, RecieverStream), Box<dyn std::error::Error>> {
        let ws_stream = match connect_async(&self.to).await {
            Ok((stream, response)) => {
                info!("Handshake for client has been completed");
                info!("Server response was {response:?}");
                stream
            }
            Err(e) => {
                error!("Websocket handshake for client failed with {e}");
                return Err(e.into());
            }
        };

        let (sender, reciever) = ws_stream.split();

        Ok((sender, reciever))
    }

    async fn request_files(
        &self,
        client: &mut SenderStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut dir = walk_dir(&self.from)?;

        let _ = dir.sort_by(|a, b| b.modified.cmp(&a.modified));

        let t = Data::WebMessage { directories: dir };

        let json_message = serde_json::to_string(&t)?;

        client.send(Message::Text(json_message)).await?;
        info!("sent message");

        Ok(())
    }
}

pub async fn run_sender(from: PathBuf, to: Url) {
    let sender = Sender::new(from, to);
    let (mut client, mut receipient) = sender.connect().await.unwrap();

    sender.request_files(&mut client).await.unwrap();

    let (tx, mut rx) = mpsc::channel(64);

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receipient.next().await {
            let vc = decode_response(message).unwrap();
            for file in vc.into_iter() {
                let tx_clone = tx.clone();
                send_json(file, tx_clone).await.unwrap();
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let (mut client, _) = sender.connect().await.unwrap();

            if let Err(e) = client.send(message).await {
                error!("client failed to send message: {e}");
            }
        }

        let (mut client, _) = sender.connect().await.unwrap();

        // close out connect to server at the end
        client
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Cow::from("Goodbye"),
            })))
            .await
            .unwrap();
    });

    recv_task.await.unwrap();
    send_task.await.unwrap();
}

async fn send_json(
    file: Directory,
    tx: tokio::sync::mpsc::Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ct = transfer_new_file(&file)?;
    let tmp = Data::JsonFileMessage { file: ct };
    let json_message = serde_json::to_string(&tmp)?;
    tx.send(Message::Text(json_message)).await.unwrap();
    Ok(())
}

fn decode_response(message: Message) -> Result<Vec<Directory>, Box<dyn std::error::Error>> {
    match message {
        Message::Text(t) => match serde_json::from_str::<Data>(&t) {
            Ok(data) => match data {
                Data::WebMessage { directories } => Ok(directories),
                Data::JsonFileMessage { .. } => {
                    Err(Box::from("recieved file contents from server"))
                }
            },
            Err(e) => Err(Box::from("Failed to parse data: {e}")),
        },
        _ => Err(Box::from("Recieved wrong message type: {_}")),
    }
}
