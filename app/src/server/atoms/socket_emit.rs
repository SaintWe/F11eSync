use super::atom_helper_messages;
use super::state as state_atoms;
use crate::proto::{ChunkAck, ChunkComplete, ChunkData, ChunkStart, DeletePayload, ServerLog, SyncControl};
use crate::server::RuntimeState;
use anyhow::Result;
use serde_json::Value;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};

pub fn send_server_warning(state: &RuntimeState, title: String, reason: String) {
    let payload = ServerLog {
        action: "server_log".to_string(),
        path: title.clone(),
        status: "warning".to_string(),
        message: Some(reason.clone()),
        content: None,
    };

    state_atoms::ui_log(
        state,
        "warn",
        atom_helper_messages::format_ts_warning_line(&reason, &title),
    );

    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("server_log", &payload);
    }
}

pub fn send_file_size_warning(state: &RuntimeState, path: String, reason: String) {
    let payload = ServerLog {
        action: "server_log".to_string(),
        path: path.clone(),
        status: "warning".to_string(),
        message: Some(reason.clone()),
        content: None,
    };

    state_atoms::ui_log(state, "warn", format!("文件过大，跳过 -> {path} ({reason})"));

    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("server_log", &payload);
    }
}

pub fn emit_update_small(state: &RuntimeState, rel: &str, b64: String) {
    let payload = serde_json::json!({
        "action": "update",
        "path": rel,
        "content": b64,
        "isDir": false,
        "encoding": "base64",
    });
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("update", &payload);
        state_atoms::ui_log(state, "info", format!("广播: update -> {rel}"));
    }
}

pub fn emit_create_dir(state: &RuntimeState, rel: &str) {
    let payload = serde_json::json!({
        "action": "create_dir",
        "path": rel,
        "content": Value::Null,
        "isDir": true,
    });
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("create_dir", &payload);
        state_atoms::ui_log(state, "info", format!("广播: create_dir -> {rel}"));
    }
}

pub fn emit_delete(state: &RuntimeState, rel: &str, is_dir: bool) {
    let payload = DeletePayload {
        action: "delete".to_string(),
        path: rel.to_string(),
        content: None,
        isDir: is_dir,
    };
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("delete", &payload);
        state_atoms::ui_log(state, "info", format!("广播: delete -> {rel}"));
    }
}

pub fn emit_sync_control(state: &RuntimeState, action: &'static str) {
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let payload = SyncControl {
            action: action.to_string(),
            path: "".to_string(),
            content: None,
            isDir: false,
        };
        let _ = socket.emit(action, &payload);
    }
}

pub fn emit_sync_error(state: &RuntimeState, msg: String) {
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let payload = SyncControl {
            action: "sync_error".to_string(),
            path: "".to_string(),
            content: Some(msg),
            isDir: false,
        };
        let _ = socket.emit("sync_error", &payload);
    }
}

pub fn emit_chunk_start(state: &RuntimeState, start: &ChunkStart) {
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("chunk_start", start);
    }
}

pub fn emit_chunk_complete(state: &RuntimeState, complete: &ChunkComplete) {
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("chunk_complete", complete);
    }
}

pub fn emit_chunk_ack(state: &RuntimeState, ack: &ChunkAck) {
    if let Some(socket) = state.socket.lock().unwrap().as_ref() {
        let _ = socket.emit("chunk_ack", ack);
    }
}

pub async fn send_chunk_and_wait_ack(
    state: &RuntimeState,
    file_id: &str,
    chunk_index: u32,
    payload: &ChunkData,
) -> Result<bool> {
    let Some(socket) = state.socket.lock().unwrap().as_ref().cloned() else {
        return Ok(false);
    };

    let (tx, rx) = oneshot::channel::<bool>();
    let key = format!("{file_id}-{chunk_index}");
    state_atoms::insert_ack_waiter(state, key.clone(), tx);

    socket.emit("chunk_data", payload).ok();

    let ok = timeout(Duration::from_secs(5), rx)
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or(false);
    if !ok {
        let _ = state_atoms::remove_ack_waiter(state, &key);
    }
    Ok(ok)
}
