use chat_demo::chat_service_client::ChatServiceClient;
use chat_demo::gui::gui;
use chat_demo::{ClientMessage, ServerMessage};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tonic::Request;
use tracing::{error, info};

const CHANNEL_SIZE: usize = 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // connect grpc
    let addr = "http://localhost:8081";
    let client = ChatServiceClient::connect(addr).await?;

    let (svr_tx, svr_rx) = mpsc::channel(CHANNEL_SIZE);
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

    let mut tasks = Vec::with_capacity(3);

    tasks.push(tokio::spawn(async {
        view(tx, svr_rx);
    }));

    tasks.push(tokio::spawn(grpc_loop(client, rx, svr_tx)));
    info!("run ...");
    info!("{:?}", futures::future::select_all(tasks).await.0);
    Ok(())
}

fn view(tx: mpsc::Sender<ClientMessage>, rx: mpsc::Receiver<ServerMessage>) {
    let mut view = gui::View::new(800, 600, "chat demo grpc client".to_string(), tx);
    info!("view show start");
    view.show(rx);
    info!("view show end");
}

async fn grpc_loop(
    mut client: ChatServiceClient<Channel>,
    rx: mpsc::Receiver<ClientMessage>,
    tx: mpsc::Sender<ServerMessage>,
) {
    info!("grpc_loop start");

    match client
        .send_message(Request::new(ReceiverStream::new(rx)))
        .await
    {
        Ok(stream) => {
            info!("grpc_loop ok");
            let mut stream = stream.into_inner();

            while let Ok(Some(res)) = stream.message().await {
                info!("recv {res:?}");
                if let Err(e) = tx.send(res).await {
                    error!("{}", e);
                }
            }
        }
        Err(e) => {
            error!("{}", e);
        }
    }
    info!("grpc_loop end");
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::io::{stdin, AsyncBufReadExt};
    use tokio::io::{AsyncBufRead, AsyncReadExt};

    #[tokio::test]
    async fn read() {
        // read terminal input line
        let mut input = String::new();
        tokio::io::BufReader::new(stdin())
            .read_line(&mut input)
            .await
            .unwrap();
        println!("{}", input);
    }
}
