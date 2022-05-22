use chat_demo::protocol::{convert_err, CERT_PEM};
use chat_demo::{ClientMessage, SendMessage, ServerMessage};
use s2n_quic::client::Connect;
use s2n_quic::Client;
use std::net::SocketAddr;

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

    let (mut recevier, mut sender) = conn.open_bidirectional_stream().await?.split();

    tokio::spawn(async move {
        let msg = ClientMessage {
            topic: "a".to_string(),
            message: Some(SendMessage("abc".to_string())),
        };
        sender.send(msg.try_into().unwrap()).await.unwrap();
    });

    while let Some(msg) = recevier.receive().await? {
        let msg: ServerMessage = msg.try_into()?;
        println!("{msg:?}");
    }

    Ok(())
}
