use crate::proto::{ChunkComplete, ChunkData, ChunkStart};
use crate::server::atoms::{atom_helper_limits, socket_emit, state as state_atoms};
use crate::server::RuntimeState;
use anyhow::Result;
use base64::Engine;
use std::path::Path;
use tokio::time::Duration;

const CHUNK_SIZE: usize = 256 * 1024;

pub async fn broadcast_create_dir(state: &RuntimeState, rel: &str) {
    if state_atoms::should_filter_rel(state, rel) {
        socket_emit::send_server_warning(
            state,
            format!("create_dir -> {rel}"),
            "匹配过滤规则，已跳过".to_string(),
        );
        return;
    }
    socket_emit::emit_create_dir(state, rel);
}

pub async fn broadcast_delete(state: &RuntimeState, rel: &str, is_dir: bool) {
    if state_atoms::should_filter_rel(state, rel) {
        socket_emit::send_server_warning(
            state,
            format!("delete -> {rel}"),
            "匹配过滤规则，已跳过".to_string(),
        );
        return;
    }
    socket_emit::emit_delete(state, rel, is_dir);
}

pub async fn broadcast_file(state: &RuntimeState, rel: &str, abs: &Path) -> Result<()> {
    if state_atoms::should_filter_rel(state, rel) {
        socket_emit::send_server_warning(
            state,
            format!("update -> {rel}"),
            "匹配过滤规则，已跳过".to_string(),
        );
        return Ok(());
    }

    let client = state.client_config.lock().unwrap().clone();
    let meta = tokio::fs::metadata(abs).await?;
    if let Some(reason) = atom_helper_limits::validate_file_size(meta.len(), &client, &state.cfg) {
        socket_emit::send_file_size_warning(state, rel.to_string(), reason);
        return Ok(());
    }

    let bytes = tokio::fs::read(abs).await?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);

    if b64.len() <= CHUNK_SIZE {
        socket_emit::emit_update_small(state, rel, b64);
        return Ok(());
    }

    let total_chunks = ((b64.len() as f64) / (CHUNK_SIZE as f64)).ceil() as u32;
    for file_retry in 0..=3 {
        if state.socket.lock().unwrap().is_none() {
            state_atoms::ui_log(state, "info", format!("客户端已断开，停止发送: {rel}"));
            return Ok(());
        }

        if file_retry > 0 {
            state_atoms::ui_log(state, "warn", format!("重试文件发送: {rel}, 第 {file_retry} 次"));
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        let file_id = format!("{}-{}", chrono_millis(), random_id());
        let start = ChunkStart {
            path: rel.to_string(),
            fileId: file_id.clone(),
            totalChunks: total_chunks,
            totalSize: Some(meta.len()),
            isDir: Some(false),
        };
        socket_emit::emit_chunk_start(state, &start);
        state_atoms::ui_log(
            state,
            "info",
            format!("开始发送分片: {rel}, {total_chunks} 个分片"),
        );

        let mut file_ok = true;
        for chunk_index in 0..total_chunks {
            if state.socket.lock().unwrap().is_none() {
                state_atoms::ui_log(state, "info", format!("客户端已断开，停止发送: {rel}"));
                file_ok = false;
                break;
            }

            let start_i = (chunk_index as usize) * CHUNK_SIZE;
            let end_i = ((chunk_index as usize + 1) * CHUNK_SIZE).min(b64.len());
            let chunk_content = b64[start_i..end_i].to_string();
            let payload = ChunkData {
                fileId: file_id.clone(),
                chunkIndex: chunk_index,
                content: chunk_content,
                path: Some(rel.to_string()),
            };

            let mut ok = false;
            for retry in 0..=3 {
                if state.socket.lock().unwrap().is_none() {
                    break;
                }
                if retry > 0 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                if socket_emit::send_chunk_and_wait_ack(state, &file_id, chunk_index, &payload)
                    .await?
                {
                    ok = true;
                    break;
                }
            }

            if !ok {
                file_ok = false;
                state_atoms::ui_log(
                    state,
                    "warn",
                    format!("分片发送失败: {rel}, chunk {chunk_index}"),
                );
                break;
            }

            if let Some(line) = crate::server::atoms::atom_helper_messages::format_chunk_progress(
                chunk_index,
                total_chunks,
                "发送分片",
                true,
            ) {
                state_atoms::ui_log(state, "info", line);
            }
        }

        if file_ok {
            let complete = ChunkComplete {
                fileId: file_id,
                path: Some(rel.to_string()),
            };
            socket_emit::emit_chunk_complete(state, &complete);
            state_atoms::ui_log(state, "info", format!("分片发送完成: {rel}"));
            break;
        }
    }

    Ok(())
}

fn chrono_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn random_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CNT: AtomicU64 = AtomicU64::new(1);
    format!("{:x}", CNT.fetch_add(1, Ordering::Relaxed))
}
