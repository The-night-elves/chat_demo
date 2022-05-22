// 单个 topic 处理

use crate::wire::ServerMessage;
use dashmap::DashSet;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::info;

const SUBSCRIPT_SIZE: usize = 16;

// global topic store

#[derive(Clone)]
pub struct Topic {
    pub id: String,
    pub subscribes: DashSet<String>,
    sequence: u64,
    input_stream: Sender<ServerMessage>,
}

impl Topic {
    pub fn new(id: String) -> Topic {
        let (tx, _) = broadcast::channel(SUBSCRIPT_SIZE);
        Topic {
            id,
            sequence: 0,
            input_stream: tx,
            subscribes: DashSet::new(),
        }
    }

    pub fn subscribe(&mut self, user_name: String) -> Receiver<ServerMessage> {
        self.subscribes.insert(user_name);
        self.input_stream.subscribe()
    }

    pub fn unsubscribe(&mut self, user_name: String) -> usize {
        self.subscribes.remove(&user_name);
        self.subscribes.len()
    }

    pub fn publish(&mut self, msg: String) -> anyhow::Result<()> {
        info!("publish message: {msg}");
        self.sequence += 1;
        let msg = ServerMessage {
            sequence: self.sequence,
            topic: self.id.clone(),
            message: Some(msg),
        };
        self.input_stream.send(msg)?;
        Ok(())
    }
}

impl Drop for Topic {
    fn drop(&mut self) {
        info!("topic drop: {}", self.id);
    }
}
