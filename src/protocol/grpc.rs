use crate::wire::chat_service_server::ChatService;
use crate::wire::{ClientMessage, ServerMessage};
use std::pin::Pin;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use tonic::codegen::futures_core::Stream;
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

const CHANNEL_SIZE: usize = 4;

pub struct ChatServer;

#[async_trait]
impl ChatService for ChatServer {
    type SendMessageStream =
        Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send + Sync + 'static>>;

    async fn send_message(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> Result<Response<Self::SendMessageStream>, Status> {
        let (tx, rx) = channel::<Result<ServerMessage, Status>>(CHANNEL_SIZE);

        tokio::spawn(async move {
            let mut stream = request.into_inner();
            while let Some(msg) = stream.message().await? {
                info!("Received message: {msg:?}");

                let res = Ok(ServerMessage {
                    sequence: 1,
                    topic: "unkown".to_string(),
                    message: msg.get_message_string(),
                });
                tx.send(res).await?;
            }

            Ok::<(), anyhow::Error>(())
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
