use crate::server::UiEvent;
use iced::widget::text_editor;
use std::path::PathBuf;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::{mpsc, oneshot};

use super::diplomat;

#[derive(Debug, Clone)]
pub(super) enum Message {
    Tick,
    CloseRequested(iced::window::Id),
    BrowseDir,
    DirInputChanged(String),
    ApplyDirInput,
    HostChanged(String),
    PortChanged(String),
    StartStop,
    CollapseLogsSidebar,
    ToggleAdvanced(bool),
    ToggleFollowSystemTheme(bool),
    ToggleDarkMode(bool),
    ServerRegexEdited(text_editor::Action),
    EnableSizeLimit(bool),
    MaxSizeChanged(String),
    CheckUpdate,
    CheckUpdateDone(String),
    DownloadUpdate,
    DownloadUpdateDone(String),
    ClearLogs,
    CopyLogs,
    ExportLogs,
    TrayMenu(String),
    ToggleFollowLogs(bool),
}

pub(super) struct BgServer {
    pub(super) shutdown: Option<oneshot::Sender<()>>,
}

impl BgServer {
    pub(super) fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

pub(super) struct F11App {
    pub(super) host: String,
    pub(super) port: String,
    pub(super) dir: PathBuf,
    pub(super) dir_input: String,
    pub(super) running: bool,
    pub(super) stopping: bool,
    pub(super) connected: bool,
    pub(super) last_error: String,

    pub(super) show_advanced: bool,
    pub(super) dark_mode: bool,
    pub(super) manual_dark_mode: bool,
    pub(super) follow_system_theme: bool,

    pub(super) server_side_regex: text_editor::Content,
    pub(super) enable_size_limit: bool,
    pub(super) max_size: String,

    pub(super) logs: Vec<String>,
    pub(super) logs_dirty: bool,
    pub(super) follow_logs: bool,
    pub(super) show_logs_sidebar: bool,

    pub(super) config_dirty: bool,
    pub(super) config_last_change: Instant,

    pub(super) server: Option<BgServer>,
    pub(super) ui_rx: mpsc::UnboundedReceiver<UiEvent>,
    pub(super) ui_tx: mpsc::UnboundedSender<UiEvent>,

    pub(super) tray: Option<diplomat::TrayHandle>,
    pub(super) minimized: bool,

    pub(super) log_scroll_id: iced::widget::scrollable::Id,

    pub(super) last_theme_check: Instant,
}

impl F11App {
    pub(super) fn detect_dark_mode() -> Option<bool> {
        let mode = dark_light::detect();
        match mode {
            dark_light::Mode::Dark => Some(true),
            dark_light::Mode::Light => Some(false),
            dark_light::Mode::Default => None,
        }
    }

    pub(super) fn refresh_theme_from_system_if_needed(&mut self) {
        if !self.follow_system_theme {
            return;
        }
        if self.last_theme_check.elapsed() < StdDuration::from_secs(1) {
            return;
        }
        self.last_theme_check = Instant::now();

        let Some(dark) = Self::detect_dark_mode() else {
            return;
        };
        if dark != self.dark_mode {
            self.dark_mode = dark;
        }
    }

    pub(super) fn touch_config(&mut self) {
        self.config_dirty = true;
        self.config_last_change = Instant::now();
    }
}
