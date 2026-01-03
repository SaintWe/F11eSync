use crate::server::atoms::{socket_emit, state as state_atoms};
use crate::server::molecules::shared_broadcast_entry;
use crate::server::RuntimeState;
use crate::watcher::{normalize_rel_path, should_ignore_rel, FsEvent, FsEventKind};
use anyhow::Result;
use tracing::warn;
use walkdir::WalkDir;

pub async fn handle_fs_event(state: &RuntimeState, ev: FsEvent) -> Result<()> {
    if state.socket.lock().unwrap().is_none() {
        return Ok(());
    }

    let Some(rel) = normalize_rel_path(&state.cfg.dir, &ev.abs_path) else { return Ok(()) };
    let rel = rel.replace('\\', "/");
    if should_ignore_rel(&rel) {
        return Ok(());
    }
    if state.server_written.lock().unwrap().contains_key(&rel) {
        return Ok(());
    }
    if state.client_written.lock().unwrap().contains_key(&rel) {
        return Ok(());
    }

    match ev.kind {
        FsEventKind::AddFile | FsEventKind::ChangeFile => {
            // notify 可能会把目录变更当作 Modify 事件，这里兜底判断
            if let Ok(meta) = tokio::fs::metadata(&ev.abs_path).await {
                if meta.is_dir() {
                    shared_broadcast_entry::broadcast_create_dir(state, &rel).await;
                    return Ok(());
                }
            }
            shared_broadcast_entry::broadcast_file(state, &rel, &ev.abs_path).await?;
        }
        FsEventKind::AddDir => {
            shared_broadcast_entry::broadcast_create_dir(state, &rel).await;

            // TS 行为：目录创建后，同时广播其当前内容
            let base = ev.abs_path.clone();
            let mut it = WalkDir::new(&base).into_iter();
            while let Some(entry) = it.next().transpose().ok().flatten() {
                if state.socket.lock().unwrap().is_none() {
                    state_atoms::ui_log(state, "info", "客户端已断开，停止目录遍历");
                    return Ok(());
                }

                let abs = entry.path().to_path_buf();
                if abs == base {
                    continue;
                }
                let Some(child_rel) = normalize_rel_path(&state.cfg.dir, &abs) else { continue };
                let child_rel = child_rel.replace('\\', "/");
                if should_ignore_rel(&child_rel) {
                    continue;
                }

                let is_dir = entry.file_type().is_dir();
                if state_atoms::should_filter_rel(state, &child_rel) {
                    let action = if is_dir { "create_dir" } else { "update" };
                    socket_emit::send_server_warning(
                        state,
                        format!("{action} -> {child_rel}"),
                        "匹配过滤规则，已跳过".to_string(),
                    );
                    if is_dir {
                        it.skip_current_dir();
                    }
                    continue;
                }

                if is_dir {
                    shared_broadcast_entry::broadcast_create_dir(state, &child_rel).await;
                } else if let Err(err) = shared_broadcast_entry::broadcast_file(state, &child_rel, &abs).await {
                    warn!("发送失败: {child_rel}: {err:#}");
                }
            }
        }
        FsEventKind::RemoveFile => {
            shared_broadcast_entry::broadcast_delete(state, &rel, false).await;
        }
        FsEventKind::RemoveDir => {
            shared_broadcast_entry::broadcast_delete(state, &rel, true).await;
        }
    }
    Ok(())
}
