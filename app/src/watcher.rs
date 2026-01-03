use crate::config::ServerConfig;
use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug, Clone)]
pub enum FsEventKind {
    AddFile,
    ChangeFile,
    AddDir,
    RemoveFile,
    RemoveDir,
}

#[derive(Debug, Clone)]
pub struct FsEvent {
    pub kind: FsEventKind,
    pub abs_path: PathBuf,
}

fn classify_event(event: &Event) -> Option<FsEventKind> {
    match &event.kind {
        EventKind::Create(kind) => match kind {
            notify::event::CreateKind::File => Some(FsEventKind::AddFile),
            notify::event::CreateKind::Folder => Some(FsEventKind::AddDir),
            _ => None,
        },
        EventKind::Modify(_) => Some(FsEventKind::ChangeFile),
        EventKind::Remove(kind) => match kind {
            notify::event::RemoveKind::File => Some(FsEventKind::RemoveFile),
            notify::event::RemoveKind::Folder => Some(FsEventKind::RemoveDir),
            _ => None,
        },
        _ => None,
    }
}

pub fn normalize_rel_path(dir: &Path, abs_path: &Path) -> Option<String> {
    let rel = abs_path.strip_prefix(dir).ok()?;
    let rel = rel.to_string_lossy().to_string();
    if rel.is_empty() {
        return None;
    }
    Some(rel.replace('\\', "/"))
}

pub fn should_ignore_rel(rel: &str) -> bool {
    rel == ".DS_Store" || rel.ends_with("/.DS_Store") || rel.ends_with(".DS_Store")
}

pub fn start_watcher(
    cfg: &ServerConfig,
) -> Result<(RecommendedWatcher, mpsc::UnboundedReceiver<FsEvent>)> {
    let (tx, rx) = mpsc::unbounded_channel::<FsEvent>();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        match res {
            Ok(event) => {
                let Some(kind) = classify_event(&event) else { return };
                for path in event.paths {
                    let _ = tx.send(FsEvent { kind: kind.clone(), abs_path: path });
                }
            }
            Err(err) => {
                warn!("监控错误: {err}");
            }
        }
    })?;

    watcher.watch(&cfg.dir, RecursiveMode::Recursive)?;
    Ok((watcher, rx))
}

