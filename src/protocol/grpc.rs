use crate::wire::chat_service_server::ChatService;
use crate::wire::{ClientMessage, ServerMessage};
use crate::{generate_uid, Session, SessionStore, TopicStore};
use futures::future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use tonic::codegen::futures_core::Stream;
use tonic::{Request, Response, Status, Streaming};
use tracing::info;
use tracing::log::error;

const CHANNEL_SIZE: usize = 4;

pub struct ChatServer {
    sessions: Arc<SessionStore>,
    topics: Arc<TopicStore>,
}

impl ChatServer {
    pub fn new(sessions: Arc<SessionStore>, topics: Arc<TopicStore>) -> Self {
        ChatServer { sessions, topics }
    }
}

#[async_trait]
impl ChatService for ChatServer {
    type SendMessageStream =
        Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send + Sync + 'static>>;

    async fn send_message(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> Result<Response<Self::SendMessageStream>, Status> {
        let (result_tx, result_rx) = channel::<Result<ServerMessage, Status>>(CHANNEL_SIZE);

        let (client_tx, client_rx) = channel(CHANNEL_SIZE);
        let (server_tx, mut server_rx) = channel(CHANNEL_SIZE);
        let id = generate_uid();
        info!("start grpc {id:?}");
        let mut sess = Session::new(id.clone(), self.topics.clone(), server_tx);
        self.sessions.add(sess.clone());

        let mut tasks = vec![];
        let sess_task =
            tokio::spawn(async move { Ok::<(), anyhow::Error>(sess.run(client_rx).await?) });
        tasks.push(sess_task);

        let task = tokio::spawn(async move {
            let mut stream = request.into_inner();
            while let Some(msg) = stream.message().await? {
                info!("Received message: {msg:?}");
                client_tx.send(msg).await?;
            }

            Ok::<(), anyhow::Error>(())
        });
        tasks.push(task);

        let task = tokio::spawn(async move {
            while let Some(msg) = server_rx.recv().await {
                info!("send message: {msg:?}");
                if let Err(e) = result_tx.send(Ok(msg)).await {
                    error!("send message error: {e}");
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        tasks.push(task);

        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            let result = future::select_all(tasks).await.0;
            info!("{id:?} disconnected {result:?}");
            sessions.remove(id);
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(result_rx))))
    }
}
