pub mod atoms;
pub mod molecules;
mod atom_api_doc;

use anyhow::{Context, Result};
use axum::routing::get;
use axum::Router;
use regex::Regex;
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::config::ServerConfig;
use crate::proto::{ChunkReceiveState, ClientConfig};
use self::molecules::fs_broadcast;

#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "gui"), allow(dead_code))]
pub enum UiEvent {
    Log { level: &'static str, message: String },
    Running { addr: SocketAddr },
    Stopped,
    ClientConnected,
    ClientDisconnected,
}

#[derive(Clone)]
pub(crate) struct RuntimeState {
    pub(crate) cfg: ServerConfig,
    pub(crate) socket: Arc<Mutex<Option<SocketRef>>>,
    pub(crate) client_config: Arc<Mutex<ClientConfig>>,
    pub(crate) server_written: Arc<Mutex<HashMap<String, u64>>>,
    pub(crate) client_written: Arc<Mutex<HashMap<String, u64>>>,
    pub(crate) chunk_receive_state: Arc<Mutex<HashMap<String, ChunkReceiveState>>>,
    pub(crate) chunk_ack_waiters: Arc<Mutex<HashMap<String, oneshot::Sender<bool>>>>,
    pub(crate) ui_tx: mpsc::UnboundedSender<UiEvent>,
    pub(crate) server_side_regex: Arc<Vec<Regex>>,
    pub(crate) effective_regex: Arc<Mutex<Vec<Regex>>>,
}

pub async fn run_headless(cfg: ServerConfig) -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown_tx.send(());
    });
    run_server(cfg, shutdown_rx, None).await
}

pub async fn run_server(
    cfg: ServerConfig,
    mut shutdown: oneshot::Receiver<()>,
    ui_tx: Option<mpsc::UnboundedSender<UiEvent>>,
) -> Result<()> {
    tokio::fs::create_dir_all(&cfg.dir)
        .await
        .with_context(|| format!("创建同步目录失败: {}", cfg.dir.display()))?;

    let ui_tx = ui_tx.unwrap_or_else(|| {
        let (tx, _rx) = mpsc::unbounded_channel::<UiEvent>();
        tx
    });

    let server_side_regex = cfg
        .server_side_path_regex
        .iter()
        .filter_map(|s| Regex::new(s).ok())
        .collect::<Vec<_>>();

    let state = RuntimeState {
        cfg: cfg.clone(),
        socket: Arc::new(Mutex::new(None)),
        client_config: Arc::new(Mutex::new(ClientConfig::default())),
        server_written: Arc::new(Mutex::new(HashMap::new())),
        client_written: Arc::new(Mutex::new(HashMap::new())),
        chunk_receive_state: Arc::new(Mutex::new(HashMap::new())),
        chunk_ack_waiters: Arc::new(Mutex::new(HashMap::new())),
        ui_tx: ui_tx.clone(),
        server_side_regex: Arc::new(server_side_regex),
        effective_regex: Arc::new(Mutex::new(Vec::new())),
    };
    *state.effective_regex.lock().unwrap() = state.server_side_regex.as_ref().clone();

    let (layer, io) = SocketIo::builder().with_state(state.clone()).build_layer();
    io.ns("/", self::molecules::socket_handlers::on_connect);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "F11eSync (Rust) is running" }))
        .layer(cors)
        .layer(layer);

    let addr = SocketAddr::new(cfg.host, cfg.port);
    info!("监听地址: http://{}:{}", cfg.host, cfg.port);
    let _ = ui_tx.send(UiEvent::Log {
        level: "info",
        message: format!("监听地址: http://{}:{}", cfg.host, cfg.port),
    });
    let _ = ui_tx.send(UiEvent::Running { addr });

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let (_watcher, mut fs_rx) = crate::watcher::start_watcher(&cfg)?;
    let state_for_fs = state.clone();
    tokio::spawn(async move {
        while let Some(ev) = fs_rx.recv().await {
            if state_for_fs.socket.lock().unwrap().is_none() {
                continue;
            }
            if let Err(err) = fs_broadcast::handle_fs_event(&state_for_fs, ev).await {
                warn!("处理文件事件失败: {err:#}");
            }
        }
    });

    tokio::select! {
        res = axum::serve(listener, app) => {
            if let Err(err) = res {
                error!("HTTP/Socket.IO 服务退出: {err}");
            }
        }
        _ = &mut shutdown => {
            info!("收到退出信号，正在关闭服务器...");
        }
    }

    if let Some(socket) = state.socket.lock().unwrap().take() {
        let _ = socket.disconnect();
    }
    let _ = ui_tx.send(UiEvent::Stopped);
    Ok(())
}
