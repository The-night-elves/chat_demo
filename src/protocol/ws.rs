use crate::session::{Session, SessionStore, TopicStore};
use crate::utils::generate_uid;
use crate::wire::ClientMessage;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::Extension;
use futures::{future, SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tracing::info;

const CHANNEL_SIZE: usize = 100;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(sessions): Extension<Arc<SessionStore>>,
    Extension(topics): Extension<Arc<TopicStore>>,
) -> impl IntoResponse {
    ws.on_upgrade(|s| async { handle_ws(s, sessions, topics).await.unwrap() })
}

pub async fn handle_ws(
    stream: WebSocket,
    sessions: Arc<SessionStore>,
    topics: Arc<TopicStore>,
) -> anyhow::Result<()> {
    // info!("{stream:?}");

    let (tx, rx) = channel(CHANNEL_SIZE);
    let (tx1, mut rx1) = channel(CHANNEL_SIZE);

    let id = generate_uid();
    let mut sess = Session::new(id.clone(), topics.clone(), tx1.clone());

    sessions.add(sess.clone());
    let mut tasks = vec![];
    let sess_task = tokio::spawn(async move { Ok::<(), anyhow::Error>(sess.run(rx).await?) });
    tasks.push(sess_task);

    let (mut sender, mut reciver) = stream.split();

    let send_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = reciver.next().await {
            match msg {
                Message::Text(msg) => {
                    info!("recive message {msg:?}");
                    let msg: ClientMessage = msg.try_into()?;
                    // send to session handler
                    tx.send(msg).await?;
                }
                Message::Close(e) => {
                    info!("Close: {e:?}");
                    return Ok(());
                }
                _ => {}
            }
        }
        Ok::<(), anyhow::Error>(())
    });
    tasks.push(send_task);

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = rx1.recv().await {
            sender.send(Message::Text(msg.try_into()?)).await?;
        }
        Ok::<(), anyhow::Error>(())
    });
    tasks.push(recv_task);

    // multi task select all
    let result = future::select_all(tasks).await.0;
    info!("{id:?} disconnected");
    sessions.remove(id);

    result?
}

#[cfg(test)]
mod test {

    // 测试多 task 结束
    #[tokio::test]
    async fn test_mult_task() {
        let t1 = tokio::spawn(async {
            loop {
                println!("t1");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        let t2 = tokio::spawn(async {
            println!("t2");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            println!("t2 end");
        });
        futures::future::select_all(vec![t1, t2]).await.0.unwrap();
    }
}
