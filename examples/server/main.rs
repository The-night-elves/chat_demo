mod config;

use crate::config::Config;
use axum::http::StatusCode;
use axum::routing::{get, get_service};
use axum::{Extension, Router};
use chat_demo::chat_service_server::ChatServiceServer;
use chat_demo::{protocol, SessionStore, TopicStore};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // parse config
    let config_path = format!("{}/examples/server/config.toml", env!("CARGO_MANIFEST_DIR"));
    let config: Config = toml::from_str(&std::fs::read_to_string(&config_path)?)?;

    info!("load config {:?}", config);

    let store = Arc::new(SessionStore::new());
    let topic_store = Arc::new(TopicStore::new());

    let router = Router::new()
        .route("/ws", get(protocol::ws_handler))
        .layer(Extension(store.clone()))
        .layer(Extension(topic_store.clone()))
        .fallback(
            get_service(
                ServeDir::new(&config.ws_config.static_dir).append_index_html_on_directories(true),
            )
            .handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        );

    // example of a route that would be handled by a different handler

    let ws_addr = config.ws_config.addr.parse()?;
    let grpc_addr = config.grpc_config.addr.parse()?;
    let quic_addr = config.quic_config.addr.clone();

    tokio::spawn(async move {
        info!("ws server start {ws_addr}");
        axum::Server::bind(&ws_addr)
            .serve(router.into_make_service())
            .await?;
        Ok::<_, anyhow::Error>(())
    });

    tokio::spawn(protocol::run(quic_addr));

    info!("grpc server start {grpc_addr}");
    let server = protocol::ChatServer::new(store, topic_store);
    tonic::transport::Server::builder()
        .add_service(ChatServiceServer::new(server))
        .serve(grpc_addr)
        .await?;
    Ok(())
}
