use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    // #[serde(default)]
    pub ws_config: WsConfig,
    // #[serde(default)]
    pub grpc_config: GrpcConfig,
    // #[serde(default)]
    pub quic_config: QuicConfig,
}

#[derive(Debug, Deserialize)]
pub struct WsConfig {
    pub addr: String,
    pub static_dir: String,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:8080".to_string(),
            static_dir: format!("{}/examples/ws_static", env!("CARGO_MANIFEST_DIR")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GrpcConfig {
    pub addr: String,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:8081".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QuicConfig {
    pub addr: String,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8433".to_string(),
        }
    }
}
