use chat_demo::chat_service_client::ChatServiceClient;
use chat_demo::{ClientMessage, SendMessage};
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "http://localhost:8081";

    let mut client = ChatServiceClient::connect(addr).await?;

    let (tx, rx) = channel(16);

    tokio::spawn(async move {
        let req = ClientMessage {
            topic: "xxx".to_string(),
            message: Some(SendMessage("hello".to_string())),
        };
        tx.send(req).await.map_err(|e| eprintln!("{e:?}")).unwrap();
    });

    let mut stream = client
        .send_message(Request::new(ReceiverStream::new(rx)))
        .await?
        .into_inner();

    while let Some(res) = stream.message().await? {
        println!("{res:?}");
    }

    Ok(())
}
