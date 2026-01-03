use crate::server::atoms::{socket_emit, state as state_atoms};
use crate::server::molecules::shared_broadcast_entry;
use crate::server::RuntimeState;
use anyhow::Result;
use tracing::warn;
use walkdir::WalkDir;

pub async fn run(state: &RuntimeState) -> Result<()> {
    if state.socket.lock().unwrap().is_none() {
        return Ok(());
    }

    state_atoms::ui_log(state, "info", "开始上传全部...");
    socket_emit::emit_sync_control(state, "sync_start");

    let base = state.cfg.dir.clone();
    let mut it = WalkDir::new(&base).into_iter();
    while let Some(entry) = it.next().transpose().ok().flatten() {
        if state.socket.lock().unwrap().is_none() {
            state_atoms::ui_log(state, "info", "客户端已断开，上传中止");
            return Ok(());
        }

        let abs = entry.path().to_path_buf();
        if abs == base {
            continue;
        }

        let Some(rel) = crate::watcher::normalize_rel_path(&base, &abs) else { continue };
        if crate::watcher::should_ignore_rel(&rel) {
            continue;
        }

        let rel = rel.replace('\\', "/");
        let is_dir = entry.file_type().is_dir();
        if state_atoms::should_filter_rel(state, &rel) {
            let action = if is_dir { "create_dir" } else { "update" };
            socket_emit::send_server_warning(
                state,
                format!("{action} -> {rel}"),
                "匹配过滤规则，已跳过".to_string(),
            );
            if is_dir {
                it.skip_current_dir();
            }
            continue;
        }

        if is_dir {
            shared_broadcast_entry::broadcast_create_dir(state, &rel).await;
        } else if let Err(err) = shared_broadcast_entry::broadcast_file(state, &rel, &abs).await {
            warn!("发送失败: {rel}: {err:#}");
        }
    }

    if state.socket.lock().unwrap().is_some() {
        socket_emit::emit_sync_control(state, "sync_complete");
        state_atoms::ui_log(state, "info", "上传全部完成");
    } else {
        state_atoms::ui_log(state, "info", "客户端已断开，上传中止");
    }
    Ok(())
}
