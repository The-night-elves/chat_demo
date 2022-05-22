// 保存单个 sessoin 和 session store

use crate::session::hub::TopicStore;
use crate::wire::client_message::Message;
use crate::wire::{ClientMessage, ServerMessage};
use dashmap::DashMap;

use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub user_name: String,
    topics: Arc<TopicStore>,
    output_stream: Sender<ServerMessage>,
    subscriptions: Arc<DashMap<String, JoinHandle<()>>>,
}

impl Session {
    pub fn new(
        id: String,
        topics: Arc<TopicStore>,
        output_stream: Sender<ServerMessage>,
    ) -> Session {
        Session {
            id,
            user_name: String::new(),
            output_stream,
            topics,
            subscriptions: Arc::new(DashMap::new()),
        }
    }

    // system send to user
    pub async fn send_message(&self, msg: ServerMessage) -> anyhow::Result<()> {
        Ok(self.output_stream.send(msg).await?)
    }

    pub async fn run(
        &mut self,
        mut input_stream: TokioReceiver<ClientMessage>,
    ) -> anyhow::Result<()> {
        while let Some(msg) = input_stream.recv().await {
            match msg.message.unwrap() {
                Message::JoinRoom(_) | Message::JoinUser(_) | Message::CreateRoom(_) => {
                    if self.subscriptions.get(&msg.topic).is_none() {
                        let receiver = self.topics.subscribe(self.user_name.clone(), &msg.topic);
                        self.spawn(&msg.topic, receiver).await;
                    }
                }
                Message::LeaveRoom(_) | Message::LeaveUser(_) => {
                    if let Some((_, sub)) = self.subscriptions.remove(&msg.topic) {
                        sub.abort();
                        self.topics.unsubscribe(self.user_name.clone(), &msg.topic);
                    }
                }
                Message::SendMessage(data) => {
                    if self.subscriptions.get(&msg.topic).is_some() {
                        self.topics.send_message(&msg.topic, data)?;
                    }
                }
                Message::Login(data) => self.user_name = data.name,
            }
        }
        Ok(())
    }

    pub async fn spawn(&mut self, topic: &str, mut msg: Receiver<ServerMessage>) {
        let sender = self.output_stream.clone();
        let handle = tokio::spawn(async move {
            while let Ok(msg) = msg.recv().await {
                if let Err(e) = sender.send(msg).await {
                    error!("{e:?}");
                    return;
                }
            }
        });
        self.subscriptions.insert(topic.to_string(), handle);
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        info!("drop session {}", self.user_name);
        for item in self.subscriptions.iter() {
            info!("'{}' remove '{}'", self.user_name, item.key());
            item.value().abort();
            self.topics.unsubscribe(self.user_name.clone(), item.key())
        }
    }
}
