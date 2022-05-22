mod wire;

pub use self::wire::{client_message::Message::SendMessage, *};
use bytes::Bytes;

// 协议

impl TryFrom<String> for ClientMessage {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}

impl TryFrom<ServerMessage> for String {
    type Error = anyhow::Error;

    fn try_from(value: ServerMessage) -> Result<Self, Self::Error> {
        Ok(serde_json::to_string(&value)?)
    }
}

impl TryFrom<Bytes> for ClientMessage {
    type Error = anyhow::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&value)?)
    }
}

impl TryFrom<Bytes> for ServerMessage {
    type Error = anyhow::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&value)?)
    }
}

impl TryFrom<ClientMessage> for Bytes {
    type Error = anyhow::Error;

    fn try_from(value: ClientMessage) -> Result<Self, Self::Error> {
        Ok(Bytes::from(serde_json::to_string(&value)?))
    }
}

impl TryFrom<ServerMessage> for Bytes {
    type Error = anyhow::Error;

    fn try_from(value: ServerMessage) -> Result<Self, Self::Error> {
        Ok(Bytes::from(serde_json::to_string(&value)?))
    }
}

impl ClientMessage {
    pub fn get_message_string(&self) -> Option<String> {
        if let Some(SendMessage(msg)) = &self.message {
            return Some(msg.clone());
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::wire::client_message::Message;
    use crate::wire::{ClientMessage, JoinRoom, Login};

    impl TryFrom<ClientMessage> for String {
        type Error = anyhow::Error;

        fn try_from(value: ClientMessage) -> Result<Self, Self::Error> {
            let string = serde_json::to_string(&value)?;
            Ok(string)
        }
    }

    #[test]
    fn encode() {
        println!("encode run");
        let message = ClientMessage {
            topic: "a".into(),
            message: Some(Message::SendMessage("hello world".into())),
        };

        let x: String = message.try_into().unwrap();
        assert_eq!(
            r#"{"topic":"a","message":{"send_message":"hello world"}}"#,
            &x
        );

        let message = ClientMessage {
            topic: "".into(),
            message: Some(Message::Login(Login {
                name: "hello world".into(),
            })),
        };

        let x: String = message.try_into().unwrap();
        assert_eq!(
            r#"{"topic":"","message":{"login":{"name":"hello world"}}}"#,
            &x
        );

        let message = ClientMessage {
            topic: "room1".into(),
            message: Some(Message::JoinRoom(JoinRoom {})),
        };

        let x: String = message.try_into().unwrap();
        assert_eq!(r#"{"topic":"room1","message":{"join_room":{}}}"#, &x);
    }

    #[test]
    fn decode() {
        let data = r#"{"topic":"a","message":{"send_message":"hello world"}}"#;
        let result: ClientMessage = data.to_string().try_into().unwrap();
        assert_eq!(
            ClientMessage {
                topic: "a".into(),
                message: Some(Message::SendMessage("hello world".into())),
            },
            result
        );
    }
}
