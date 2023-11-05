use amanita_common::{find_changed_files, replace_files, Data};
use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router, Server,
};

use env_logger::Builder;
use log::{error, info};
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct Reciever {
    pub addr: SocketAddr,
    pub output_dir: PathBuf,
    pub tx: tokio::sync::mpsc::Sender<()>,
}

impl Reciever {
    fn new(port: String, output_dir: PathBuf, tx: tokio::sync::mpsc::Sender<()>) -> Self {
        Builder::new()
            .filter_level(log::LevelFilter::Info)
            .parse_env("LOG")
            .init();

        let localhost = format!("0.0.0.0:{}", port);
        let addr: SocketAddr = localhost.parse().expect("failed to parse address supplied");

        Self {
            addr,
            output_dir,
            tx,
        }
    }
}

#[debug_handler]
async fn handler(ws: WebSocketUpgrade, State(state): State<Reciever>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Reciever) {
    if let Some(Ok(message)) = socket.recv().await {
        match message {
            Message::Text(t) => match serde_json::from_str::<Data>(&t) {
                Ok(data) => match data {
                    Data::WebMessage { directories } => {
                        let new_dir = find_changed_files(directories, state.output_dir).unwrap();
                        let t = Data::WebMessage {
                            directories: new_dir,
                        };
                        let to_json = serde_json::to_string(&t).unwrap();
                        info!("sending out updated files list: {:?}", &to_json);
                        socket.send(Message::Text(to_json)).await.unwrap();
                    }

                    Data::JsonFileMessage { file } => {
                        replace_files(file, state.output_dir).unwrap();
                    }
                },
                Err(e) => {
                    error!("couldn't parse message from client: {e}");
                    return;
                }
            },
            Message::Close(t) => {
                if let Some(tf) = t {
                    info!("shutting down server {:?} for {:?}", tf.code, tf.reason);
                    let _ = state.tx.send(()).await;
                }
            }
            _ => {}
        }
    }
}

pub async fn run_reciever(port: String, output_dir: PathBuf) {
    let (tx, mut rx) = mpsc::channel(1);

    let reciever = Reciever::new(port, output_dir, tx.clone());

    let app = Router::new()
        .route("/ws", get(handler))
        .with_state(reciever.clone());

    info!("Server listening on {}", reciever.addr);

    let server = Server::bind(&reciever.addr).serve(app.into_make_service());

    let graceful = server.with_graceful_shutdown(async move {
        let _ = rx.recv().await;
    });

    // Await the `server` receiving the signal...
    if let Err(e) = graceful.await {
        error!("server error: {}", e);
    }
}
