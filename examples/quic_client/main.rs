use chat_demo::gui::gui;
use chat_demo::protocol::{convert_err, CERT_PEM};
use chat_demo::{ClientMessage, ServerMessage};
use s2n_quic::client::Connect;
use s2n_quic::stream::{ReceiveStream, SendStream};
use s2n_quic::Client;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::info;

const CHANNEL_SIZE: usize = 8;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::builder()
        .with_tls(CERT_PEM)?
        .with_io("0.0.0.0:0")?
        .start()
        .map_err(convert_err)?;

    let addr: SocketAddr = "127.0.0.1:8433".parse()?;

    let mut conn = client
        .connect(Connect::new(addr).with_server_name("localhost"))
        .await?;

    conn.keep_alive(true).map_err(convert_err)?;

    let (recevier, sender) = conn.open_bidirectional_stream().await?.split();
    let (client_tx, client_rx) = mpsc::channel(CHANNEL_SIZE);
    let (server_tx, server_rx) = mpsc::channel(CHANNEL_SIZE);
    let mut tasks = Vec::with_capacity(3);
    // gui loop
    tasks.push(tokio::spawn(async move {
        view(client_tx, server_rx);
        Ok(())
    }));
    // read loop
    tasks.push(tokio::spawn(read_loop(recevier, server_tx)));
    // write loop
    tasks.push(tokio::spawn(write_loop(sender, client_rx)));
    info!("run ...");
    // tasks select_all
    let result = futures::future::select_all(tasks).await.0;

    result?
}

fn view(tx: mpsc::Sender<ClientMessage>, rx: mpsc::Receiver<ServerMessage>) {
    let mut view = gui::View::new(800, 600, "chat demo quic client".to_string(), tx);
    info!("view show start");
    view.show(rx);
    info!("view show end");
}

async fn write_loop(
    mut sender: SendStream,
    mut rx: mpsc::Receiver<ClientMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await {
        sender.send(msg.try_into()?).await?;
    }
    Ok(())
}

async fn read_loop(
    mut receiver: ReceiveStream,
    tx: mpsc::Sender<ServerMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = receiver.receive().await? {
        let msg: ServerMessage = msg.try_into()?;
        tx.send(msg).await?;
    }
    Ok(())
}
