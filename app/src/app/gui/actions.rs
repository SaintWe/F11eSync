use crate::server::{self, UiEvent};
use crate::update;
use iced::{Command, Size};
use tokio::sync::oneshot;

use super::atom_helper_log;
use super::atom_helper_path;
use super::data_officer;
use super::{diplomat, BgServer, F11App, Message};

pub fn ensure_logs_sidebar_visible(app: &mut F11App) -> Option<Command<Message>> {
    if app.show_logs_sidebar {
        return None;
    }
    app.show_logs_sidebar = true;
    Some(iced::window::resize(
        iced::window::Id::MAIN,
        Size::new(780.0, 480.0),
    ))
}

pub fn copy_logs(app: &mut F11App) -> Command<Message> {
    if app.logs.is_empty() {
        return Command::none();
    }

    iced::clipboard::write(app.logs.join("\n"))
}

pub fn export_logs(app: &mut F11App) -> Command<Message> {
    if app.logs.is_empty() {
        return Command::none();
    }

    let Some(path) = diplomat::save_log_file(&app.dir) else {
        return Command::none();
    };

    let mut text = app.logs.join("\n");
    if !text.ends_with('\n') {
        text.push('\n');
    }

    let _ = std::fs::write(&path, text);

    Command::none()
}

pub fn push_log(app: &mut F11App, line: impl Into<String>) {
    let Some(line) = atom_helper_log::normalize_log_line(line) else {
        return;
    };

    app.logs.push(line);
    app.logs_dirty = true;
    if app.logs.len() > 500 {
        app.logs.drain(0..app.logs.len().saturating_sub(500));
    }
}

pub fn handle_ui_event(app: &mut F11App, ev: UiEvent) {
    match ev {
        UiEvent::Log { level, message } => {
            push_log(app, format!("[{level}] {message}"));
        }
        UiEvent::Running { addr } => {
            app.running = true;
            app.stopping = false;
            push_log(app, format!("[info] 服务器已启动: http://{addr}"));
        }
        UiEvent::Stopped => {
            app.running = false;
            app.stopping = false;
            app.server = None;
            app.connected = false;
            push_log(app, "[warn] 服务器已停止".to_string());
            app.touch_config();
        }
        UiEvent::ClientConnected => {
            app.connected = true;
            push_log(app, "[info] 客户端已连接".to_string());
        }
        UiEvent::ClientDisconnected => {
            app.connected = false;
            push_log(app, "[warn] 客户端已断开".to_string());
        }
    }
}

pub fn browse_dir(app: &mut F11App) -> Command<Message> {
    if let Some(p) = diplomat::pick_folder(&app.dir) {
        app.dir = std::fs::canonicalize(&p).unwrap_or(p);
        app.dir_input = atom_helper_path::display_path(&app.dir);
        app.last_error.clear();
        push_log(app, format!("[info] 选择目录: {}", atom_helper_path::display_path(&app.dir)));
        app.touch_config();
    }
    Command::none()
}

pub fn apply_dir_input(app: &mut F11App) {
    let trimmed = app.dir_input.trim();
    if trimmed.is_empty() {
        app.last_error = "目录不能为空".to_string();
        return;
    }
    let p = std::path::PathBuf::from(trimmed);
    match diplomat::ensure_dir_and_canonicalize(&p) {
        Ok(dir) => {
            app.dir = dir;
            app.last_error.clear();
            push_log(app, format!("[info] 使用目录: {}", atom_helper_path::display_path(&app.dir)));
            app.dir_input = atom_helper_path::display_path(&app.dir);
            app.touch_config();
        }
        Err(err) => {
            app.last_error = format!("创建目录失败: {err}");
        }
    }
}

pub fn persist_config_if_valid(app: &mut F11App) {
    let Some(cfg) = data_officer::build_server_config(
        &app.host,
        &app.port,
        &app.dir,
        &app.server_side_regex,
        app.enable_size_limit,
        &app.max_size,
    ) else {
        return;
    };

    let app_cfg = data_officer::build_app_config(
        &cfg,
        &app.server_side_regex,
        app.follow_system_theme,
        app.manual_dark_mode,
        app.show_advanced,
        app.follow_logs,
    );

    if let Err(err) = diplomat::save_settings(&app_cfg) {
        push_log(app, format!("[warn] 保存配置失败: {err}"));
    }
}

pub fn quit(app: &mut F11App) -> Command<Message> {
    persist_config_if_valid(app);
    if let Some(server) = &mut app.server {
        server.stop();
    }
    std::process::exit(0);
}

pub fn start_stop(app: &mut F11App) -> Command<Message> {
    app.last_error.clear();
    if app.stopping {
        push_log(app, "[warn] 正在停止服务，请稍候…".to_string());
        return Command::none();
    }
    if app.running {
        if let Some(server) = &mut app.server {
            server.stop();
        }
        app.running = false;
        app.stopping = true;
        app.connected = false;
        push_log(app, "[warn] 请求停止服务器...".to_string());
        return Command::none();
    }

    if app.dir_input.trim().is_empty() {
        app.last_error = "请先选择同步目录".to_string();
        return Command::none();
    }
    if app.dir_input.trim() != atom_helper_path::display_path(&app.dir) {
        apply_dir_input(app);
        if !app.last_error.is_empty() {
            return Command::none();
        }
    }

    let Some(cfg) = data_officer::build_server_config(
        &app.host,
        &app.port,
        &app.dir,
        &app.server_side_regex,
        app.enable_size_limit,
        &app.max_size,
    ) else {
        app.last_error = "配置不合法（Host/Port/目录）".to_string();
        return Command::none();
    };

    let mut commands: Vec<Command<Message>> = Vec::new();
    if let Some(cmd) = ensure_logs_sidebar_visible(app) {
        commands.push(cmd);
    }

    let ui_tx = app.ui_tx.clone();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new();
        match rt {
            Ok(rt) => {
                let _ = rt.block_on(server::run_server(cfg, shutdown_rx, Some(ui_tx)));
            }
            Err(err) => {
                let _ = ui_tx.send(UiEvent::Log {
                    level: "error",
                    message: format!("启动 tokio runtime 失败: {err}"),
                });
            }
        }
    });

    app.server = Some(BgServer {
        shutdown: Some(shutdown_tx),
    });
    app.running = true;
    push_log(app, "[info] 正在启动服务器...".to_string());
    Command::batch(commands)
}

pub fn collapse_logs_sidebar(app: &mut F11App) -> Command<Message> {
    if app.show_logs_sidebar {
        app.show_logs_sidebar = false;
        app.logs.clear();
        app.logs_dirty = false;
        app.touch_config();
        return iced::window::resize(iced::window::Id::MAIN, Size::new(400.0, 480.0));
    }
    Command::none()
}

pub fn check_update(app: &mut F11App) -> Command<Message> {
    push_log(app, "[info] 正在检查更新...".to_string());

    let mut commands: Vec<Command<Message>> = Vec::new();
    if let Some(cmd) = ensure_logs_sidebar_visible(app) {
        commands.push(cmd);
    }
    commands.push(Command::perform(
        async {
            update::check_update_message(false)
                .await
                .unwrap_or_else(|e| format!("检查更新失败: {e}"))
        },
        Message::CheckUpdateDone,
    ));
    Command::batch(commands)
}

pub fn download_update(app: &mut F11App) -> Command<Message> {
    push_log(app, "[info] 正在下载更新...".to_string());
    let mut commands: Vec<Command<Message>> = Vec::new();
    if let Some(cmd) = ensure_logs_sidebar_visible(app) {
        commands.push(cmd);
    }
    commands.push(Command::perform(
        async {
            update::download_update_message()
                .await
                .unwrap_or_else(|e| format!("下载更新失败: {e}"))
        },
        Message::DownloadUpdateDone,
    ));
    Command::batch(commands)
}

pub fn poll_tray_event_id() -> Option<String> {
    diplomat::try_recv_tray_event_id()
}
