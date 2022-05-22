use crate::session::topic::Topic;
use crate::session::Session;
use crate::wire::ServerMessage;
use dashmap::DashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::broadcast::Receiver;
use tracing::info;

#[derive(Clone)]
// key: topic_id, value: topic
pub struct TopicStore(DashMap<String, Topic>);

impl TopicStore {
    pub fn new() -> TopicStore {
        TopicStore(DashMap::new())
    }

    pub fn subscribe(&self, user_name: String, topic_id: &str) -> Receiver<ServerMessage> {
        match self.0.get_mut(topic_id) {
            None => {
                let mut topic = Topic::new(topic_id.into());
                let res = topic.subscribe(user_name);
                self.0.insert(topic_id.into(), topic);
                res
            }
            Some(mut topic) => topic.subscribe(user_name),
        }
    }

    pub fn unsubscribe(&self, user_name: String, topic_id: &str) {
        info!("unsubscribe topic: {}, user: {}", topic_id, user_name);
        let mut deleted = false;
        if let Some(mut topic) = self.0.get_mut(topic_id) {
            deleted = topic.unsubscribe(user_name) <= 0
        }
        if deleted {
            self.0.remove(topic_id);
        }
    }

    pub fn send_message(&self, topic_id: &str, message: String) -> anyhow::Result<()> {
        match self.0.get_mut(topic_id) {
            None => Err(anyhow::anyhow!("topic not found: {topic_id}")),
            Some(mut topic) => topic.publish(message),
        }
    }
}

impl Display for TopicStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SessionStore: {{\n")?;
        for item in self.0.iter() {
            write!(
                f,
                "  {}: s_id {} s_name {:?}\n",
                item.key(),
                item.value().id,
                item.value().subscribes,
            )?;
        }
        write!(f, "}}")
    }
}

pub struct SessionStore {
    // key: session_id, value: session
    sessions: DashMap<String, Session>,
}

impl SessionStore {
    pub fn new() -> Self {
        SessionStore {
            sessions: DashMap::new(),
        }
    }

    pub fn add(&self, sess: Session) {
        if self.sessions.get(&sess.id).is_none() {
            self.sessions.insert(sess.id.clone(), sess);
        }
    }

    pub fn remove(&self, sess_id: String) -> Option<(String, Session)> {
        self.sessions.remove(&sess_id)
    }
}

impl Display for SessionStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SessionStore: {{\n")?;
        for item in self.sessions.iter() {
            write!(
                f,
                "  {}: s_id {} s_name {}\n",
                item.key(),
                item.value().id,
                item.value().user_name,
            )?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::session::hub::TopicStore;
    use crate::wire::ServerMessage;

    #[tokio::test]
    async fn topic_store_subscribe() {
        let store = TopicStore::new();

        let topic_id = "topic_id";
        let user_name = "user_name";

        let mut res = store.subscribe(user_name.into(), topic_id);

        store.send_message(topic_id, "xxx".to_string()).unwrap();

        let result = res.recv().await.unwrap();
        assert_eq!(
            result,
            ServerMessage {
                sequence: 1,
                topic: "topic_id".to_string(),
                message: Some("xxx".to_string())
            }
        );
    }

    #[test]
    fn topics_drop() {
        let store = TopicStore::new();
        let topic_id = "topic_id";
        let user_name = "user_name";
        store.subscribe(user_name.into(), topic_id);
        store.unsubscribe(user_name.into(), topic_id);
    }
}
