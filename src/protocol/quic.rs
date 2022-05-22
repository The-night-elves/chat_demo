use crate::wire::{ClientMessage, ServerMessage};
use s2n_quic::Server;
use tracing::info;

pub fn convert_err<E: std::error::Error>(err: E) -> anyhow::Error {
    anyhow::anyhow!(err.to_string())
}

pub static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/certs/cert.pem"));

static KEY_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/certs/key.pem"));

pub async fn run(addr: String) -> anyhow::Result<()> {
    let mut server = Server::builder()
        .with_tls((CERT_PEM, KEY_PEM))?
        .with_io(addr.as_ref())?
        .start()
        .map_err(convert_err)?;

    info!("quic server start {addr:?}");

    while let Some(mut conn) = server.accept().await {
        info!("new connection from {}", conn.remote_addr()?);
        tokio::spawn(async move {
            while let Ok(Some(mut stream)) = conn.accept_bidirectional_stream().await {
                info!(
                    "new bidirectional stream from id {}",
                    stream.connection().id()
                );
                tokio::spawn(async move {
                    while let Ok(Some(msg)) = stream.receive().await {
                        info!("received {msg:?}");
                        let msg: ClientMessage = msg.try_into()?;
                        let msg = ServerMessage {
                            sequence: 1,
                            topic: "".to_string(),
                            message: msg.get_message_string(),
                        };
                        stream.send(msg.try_into()?).await?;
                    }

                    Ok::<(), anyhow::Error>(())
                });
            }
        });
    }

    Ok(())
}
