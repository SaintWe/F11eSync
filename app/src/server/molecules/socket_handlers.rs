use serde_json::Value;
use socketioxide::extract::{Data, SocketRef, State};
use tracing::error;

use crate::proto::{
    ChunkAck, ChunkComplete, ChunkData, ChunkStart, ClientConfig, ConnectionRejected, CreateDir, UpdateFile,
};

use super::{client_upload, sync_all};
use crate::server::atoms::{socket_emit, state as state_atoms};
use crate::server::RuntimeState;

pub(crate) fn on_connect(socket: SocketRef, Data(data): Data<Value>, State(state): State<RuntimeState>) {
    let _ = data;

    if !state_atoms::set_socket_if_empty(&state, socket.clone()) {
        let payload = ConnectionRejected {
            message: "连接失败：已有其他客户端连接，不允许多个客户端同时连接".to_string(),
        };
        socket.emit("connection_rejected", &payload).ok();
        socket.disconnect().ok();
        return;
    }

    state_atoms::ui_log(&state, "info", "客户端连接");
    let _ = state.ui_tx.send(crate::server::UiEvent::ClientConnected);

    socket.on("configure", |Data(v): Data<Value>, State(state): State<RuntimeState>| {
        let v = state_atoms::extract_first_arg(v);
        if let Ok(cfg) = serde_json::from_value::<ClientConfig>(v) {
            let merged = {
                let mut guard = state.client_config.lock().unwrap();
                state_atoms::merge_client_config(&mut guard, cfg.clone());
                guard.clone()
            };
            state_atoms::rebuild_effective_regex(&state, &merged);
            state_atoms::ui_log(&state, "info", format!("更新客户端配置: {:?}", cfg));
        }
    });

    socket.on("sync_all", |State(state): State<RuntimeState>| async move {
        state_atoms::ui_log(&state, "info", "收到客户端下载请求：sync_all");
        if let Err(err) = sync_all::run(&state).await {
            error!("sync_all 失败: {err:#}");
            state_atoms::ui_log(&state, "error", format!("上传全部失败: {err:#}"));
            socket_emit::emit_sync_error(&state, err.to_string());
        }
    });

    socket.on("client_upload_start", |State(state): State<RuntimeState>| {
        state_atoms::ui_log(&state, "info", "客户端开始上传全部文件...");
    });

    socket.on("client_upload_complete", |State(state): State<RuntimeState>| {
        state_atoms::ui_log(&state, "info", "客户端上传全部完成");
    });

    socket.on("update", |Data(v): Data<Value>, State(state): State<RuntimeState>| async move {
        let v = state_atoms::extract_first_arg(v);
        let Ok(data) = serde_json::from_value::<UpdateFile>(v) else {
            return;
        };
        client_upload::handle_update(&state, data).await;
    });

    socket.on("create_dir", |Data(v): Data<Value>, State(state): State<RuntimeState>| async move {
        let v = state_atoms::extract_first_arg(v);
        let Ok(data) = serde_json::from_value::<CreateDir>(v) else {
            return;
        };
        client_upload::handle_create_dir(&state, data).await;
    });

    socket.on("chunk_start", |Data(v): Data<Value>, State(state): State<RuntimeState>| {
        let v = state_atoms::extract_first_arg(v);
        let Ok(data) = serde_json::from_value::<ChunkStart>(v) else {
            return;
        };
        client_upload::handle_chunk_start(&state, data);
    });

    socket.on("chunk_data", |Data(v): Data<Value>, State(state): State<RuntimeState>| async move {
        let v = state_atoms::extract_first_arg(v);
        let Ok(data) = serde_json::from_value::<ChunkData>(v) else {
            return;
        };
        client_upload::handle_chunk_data(&state, data).await;
    });

    socket.on("chunk_complete", |Data(v): Data<Value>, State(state): State<RuntimeState>| {
        let v = state_atoms::extract_first_arg(v);
        let Ok(data) = serde_json::from_value::<ChunkComplete>(v) else {
            return;
        };
        client_upload::handle_chunk_complete(&state, data);
    });

    socket.on("chunk_ack", |Data(v): Data<Value>, State(state): State<RuntimeState>| {
        let v = state_atoms::extract_first_arg(v);
        let Ok(ack) = serde_json::from_value::<ChunkAck>(v) else {
            return;
        };
        client_upload::handle_chunk_ack(&state, ack);
    });

    socket.on_disconnect(|_socket: SocketRef, State(state): State<RuntimeState>| async move {
        state_atoms::ui_log(&state, "warn", "客户端断开连接");
        let _ = state.ui_tx.send(crate::server::UiEvent::ClientDisconnected);
        client_upload::disconnect_cleanup(&state);
    });
}
