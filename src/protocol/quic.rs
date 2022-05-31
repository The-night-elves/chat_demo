use crate::wire::{ClientMessage, ServerMessage};
use crate::{generate_uid, Session, SessionStore, TopicStore};
use s2n_quic::stream::{BidirectionalStream, ReceiveStream, SendStream};
use s2n_quic::Server;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use tracing::log::error;

pub fn convert_err<E: std::error::Error>(err: E) -> anyhow::Error {
    anyhow::anyhow!(err.to_string())
}

pub static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/certs/cert.pem"));

static KEY_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/certs/key.pem"));

const CHANNEL_SIZE: usize = 4;

pub async fn run(
    addr: String,
    sessions: Arc<SessionStore>,
    topics: Arc<TopicStore>,
) -> anyhow::Result<()> {
    let mut server = Server::builder()
        .with_tls((CERT_PEM, KEY_PEM))?
        .with_io(addr.as_ref())?
        .start()
        .map_err(convert_err)?;

    info!("quic server start {addr:?}");

    while let Some(mut conn) = server.accept().await {
        info!("new connection from {}", conn.remote_addr()?);
        let sessions = sessions.clone();
        let topics = topics.clone();
        tokio::spawn(async move {
            while let Ok(Some(stream)) = conn.accept_bidirectional_stream().await {
                info!(
                    "new bidirectional stream from id {}",
                    stream.connection().id()
                );
                let sessions = sessions.clone();
                let topics = topics.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle(stream, sessions, topics).await {
                        error!("handle error: {:?}", e);
                    };
                });
            }
        });
    }

    Ok(())
}

async fn handle(
    stream: BidirectionalStream,
    sessions: Arc<SessionStore>,
    topics: Arc<TopicStore>,
) -> anyhow::Result<()> {
    let (rx_stream, tx_stream) = stream.split();

    let (client_tx, client_rx) = mpsc::channel(CHANNEL_SIZE);
    let (server_tx, server_rx) = mpsc::channel(CHANNEL_SIZE);
    let id = generate_uid();
    info!("start grpc {id:?}");
    let mut sess = Session::new(id.clone(), topics.clone(), server_tx);
    sessions.add(sess.clone());
    let mut tasks = Vec::with_capacity(3);
    // session run
    tasks.push(tokio::spawn(async move {
        Ok::<(), anyhow::Error>(sess.run(client_rx).await?)
    }));
    // read loop
    tasks.push(tokio::spawn(read_loop(rx_stream, client_tx)));
    // write loop
    tasks.push(tokio::spawn(write_loop(tx_stream, server_rx)));
    // select all tasks
    let result = futures::future::select_all(tasks).await.0?;
    // leave info log
    info!("{id:?} disconnected {result:?}");
    sessions.remove(id);

    result
}

async fn read_loop(
    mut stream: ReceiveStream,
    tx: mpsc::Sender<ClientMessage>,
) -> anyhow::Result<()> {
    while let Ok(Some(msg)) = stream.receive().await {
        let msg: ClientMessage = msg.try_into()?;
        info!("received {msg:?}");
        tx.send(msg).await?;
    }
    Ok(())
}

async fn write_loop(
    mut stream: SendStream,
    mut rx: mpsc::Receiver<ServerMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await {
        info!("send {msg:?}");
        stream.send(msg.try_into()?).await?;
    }
    Ok(())
}
