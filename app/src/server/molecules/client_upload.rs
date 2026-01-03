use crate::proto::{ChunkAck, ChunkComplete, ChunkData, ChunkReceiveState, ChunkStart, CreateDir, UpdateFile};
use crate::server::atoms::{atom_helper_limits, socket_emit, state as state_atoms};
use crate::server::RuntimeState;
use base64::Engine;
use tracing::error;

pub async fn handle_update(state: &RuntimeState, data: UpdateFile) {
    if data.encoding.as_deref() != Some("base64") {
        return;
    }
    let rel = data.path.replace('\\', "/");
    if crate::watcher::should_ignore_rel(&rel) {
        return;
    }

    let abs = state.cfg.dir.join(&rel);
    if let Some(parent) = abs.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data.content) else {
        return;
    };
    let client = state.client_config.lock().unwrap().clone();
    if let Some(reason) = atom_helper_limits::validate_file_size(bytes.len() as u64, &client, &state.cfg) {
        socket_emit::send_file_size_warning(state, rel.clone(), reason);
        return;
    }
    if let Err(err) = tokio::fs::write(&abs, bytes).await {
        error!("写入失败: {rel}: {err}");
        return;
    }
    state_atoms::mark_client_written(state, &rel);
    state_atoms::ui_log(state, "info", format!("客户端上传文件: {rel}"));
}

pub async fn handle_create_dir(state: &RuntimeState, data: CreateDir) {
    let rel = data.path.replace('\\', "/");
    if crate::watcher::should_ignore_rel(&rel) {
        return;
    }
    let abs = state.cfg.dir.join(&rel);
    if let Err(err) = tokio::fs::create_dir_all(&abs).await {
        error!("创建目录失败: {rel}: {err}");
        return;
    }
    state_atoms::mark_client_written(state, &rel);
    state_atoms::ui_log(state, "info", format!("客户端创建目录: {rel}"));
}

pub fn handle_chunk_start(state: &RuntimeState, data: ChunkStart) {
    let rel = data.path.replace('\\', "/");
    let abs = state.cfg.dir.join(&rel);
    let client = state.client_config.lock().unwrap().clone();
    let reject_reason = data
        .totalSize
        .and_then(|sz| atom_helper_limits::validate_file_size(sz, &client, &state.cfg));
    state.chunk_receive_state.lock().unwrap().insert(
        data.fileId.clone(),
        ChunkReceiveState {
            abs_path: abs,
            rel_path: rel,
            received_chunks: 0,
            total_chunks: data.totalChunks,
            reject_reason,
        },
    );
    state_atoms::ui_log(
        state,
        "info",
        format!("开始接收分片: {}, 总分片数: {}", data.path, data.totalChunks),
    );
}

pub async fn handle_chunk_data(state: &RuntimeState, data: ChunkData) {
    let (abs_path, rel_path) = {
        let map = state.chunk_receive_state.lock().unwrap();
        let Some(st) = map.get(&data.fileId) else {
            let ack = ChunkAck {
                fileId: data.fileId,
                chunkIndex: data.chunkIndex,
                success: Some(false),
                error: Some("未找到接收状态".to_string()),
            };
            socket_emit::emit_chunk_ack(state, &ack);
            return;
        };
        (st.abs_path.clone(), st.rel_path.clone())
    };

    {
        let mut map = state.chunk_receive_state.lock().unwrap();
        if let Some(st) = map.get_mut(&data.fileId) {
            if let Some(reason) = st.reject_reason.clone() {
                if st.received_chunks == 0 {
                    socket_emit::send_server_warning(state, st.rel_path.clone(), reason.clone());
                }
                let ack = ChunkAck {
                    fileId: data.fileId,
                    chunkIndex: data.chunkIndex,
                    success: Some(false),
                    error: Some(reason),
                };
                socket_emit::emit_chunk_ack(state, &ack);
                return;
            }
        }
    }

    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&data.content) else {
        let ack = ChunkAck {
            fileId: data.fileId,
            chunkIndex: data.chunkIndex,
            success: Some(false),
            error: Some("Base64 解码失败".to_string()),
        };
        socket_emit::emit_chunk_ack(state, &ack);
        return;
    };

    if let Some(parent) = abs_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let write_res = if data.chunkIndex == 0 {
        tokio::fs::write(&abs_path, &bytes).await
    } else {
        use tokio::io::AsyncWriteExt;
        let f = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&abs_path)
            .await;
        match f {
            Ok(mut f) => f.write_all(&bytes).await,
            Err(e) => Err(e),
        }
    };

    if let Err(err) = write_res {
        let ack = ChunkAck {
            fileId: data.fileId,
            chunkIndex: data.chunkIndex,
            success: Some(false),
            error: Some(err.to_string()),
        };
        socket_emit::emit_chunk_ack(state, &ack);
        return;
    }

    {
        let mut map = state.chunk_receive_state.lock().unwrap();
        if let Some(st) = map.get_mut(&data.fileId) {
            st.received_chunks += 1;

            if let Some(line) = crate::server::atoms::atom_helper_messages::format_chunk_progress(
                st.received_chunks,
                st.total_chunks,
                "接收分片",
                false,
            ) {
                state_atoms::ui_log(state, "info", line);
            }
        }
    }
    state_atoms::mark_client_written(state, &rel_path);

    let ack = ChunkAck {
        fileId: data.fileId,
        chunkIndex: data.chunkIndex,
        success: Some(true),
        error: None,
    };
    socket_emit::emit_chunk_ack(state, &ack);
}

pub fn handle_chunk_complete(state: &RuntimeState, data: ChunkComplete) {
    if let Some(st) = state.chunk_receive_state.lock().unwrap().remove(&data.fileId) {
        state_atoms::mark_client_written(state, &st.rel_path);
        state_atoms::ui_log(
            state,
            "info",
            format!(
                "客户端上传分片完成: {:?}, {} 个分片",
                data.path, st.received_chunks
            ),
        );
        return;
    }
    state_atoms::ui_log(state, "info", format!("客户端上传分片完成: {:?}", data.path));
}

pub fn handle_chunk_ack(state: &RuntimeState, ack: ChunkAck) {
    let key = format!("{}-{}", ack.fileId, ack.chunkIndex);
    if let Some(tx) = state_atoms::remove_ack_waiter(state, &key) {
        let _ = tx.send(ack.success.unwrap_or(true));
    }
}

pub fn disconnect_cleanup(state: &RuntimeState) {
    state_atoms::clear_socket(state);
    state_atoms::reset_connection_state(state);
}
