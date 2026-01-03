use iced::widget::scrollable::RelativeOffset;
use iced::Command;
use std::time::Duration as StdDuration;

use super::{actions, F11App, Message};

pub fn update(app: &mut F11App, message: Message) -> Command<Message> {
    match message {
        Message::Tick => {
            let mut commands: Vec<Command<Message>> = Vec::new();
            app.refresh_theme_from_system_if_needed();
            while let Ok(ev) = app.ui_rx.try_recv() {
                actions::handle_ui_event(app, ev);
            }
            while let Some(id) = actions::poll_tray_event_id() {
                commands.push(update(app, Message::TrayMenu(id)));
            }

            if app.config_dirty && app.config_last_change.elapsed() > StdDuration::from_millis(600)
            {
                app.config_dirty = false;
                actions::persist_config_if_valid(app);
            }

            if app.logs_dirty && app.follow_logs && app.show_logs_sidebar {
                app.logs_dirty = false;
                commands.push(iced::widget::scrollable::snap_to(
                    app.log_scroll_id.clone(),
                    RelativeOffset { x: 0.0, y: 1.0 },
                ));
            }

            Command::batch(commands)
        }
        Message::CloseRequested(id) => {
            if id == iced::window::Id::MAIN {
                return actions::quit(app);
            }

            Command::none()
        }
        Message::BrowseDir => {
            actions::browse_dir(app)
        }
        Message::DirInputChanged(v) => {
            app.dir_input = v;
            app.touch_config();
            Command::none()
        }
        Message::ApplyDirInput => {
            actions::apply_dir_input(app);
            Command::none()
        }
        Message::HostChanged(v) => {
            app.host = v;
            app.touch_config();
            Command::none()
        }
        Message::PortChanged(v) => {
            app.port = v;
            app.touch_config();
            Command::none()
        }
        Message::EnableSizeLimit(v) => {
            app.enable_size_limit = v;
            app.touch_config();
            Command::none()
        }
        Message::ToggleAdvanced(v) => {
            app.show_advanced = v;
            app.touch_config();
            Command::none()
        }
        Message::ToggleFollowSystemTheme(v) => {
            app.follow_system_theme = v;
            if v {
                if let Some(dark) = F11App::detect_dark_mode() {
                    app.dark_mode = dark;
                }
            } else {
                app.dark_mode = app.manual_dark_mode;
            }
            app.touch_config();
            Command::none()
        }
        Message::ToggleDarkMode(v) => {
            app.manual_dark_mode = v;
            if !app.follow_system_theme {
                app.dark_mode = v;
            }
            app.touch_config();
            Command::none()
        }
        Message::MaxSizeChanged(v) => {
            app.max_size = v;
            app.touch_config();
            Command::none()
        }
        Message::ServerRegexEdited(action) => {
            app.server_side_regex.perform(action);
            app.touch_config();
            Command::none()
        }
        Message::StartStop => {
            actions::start_stop(app)
        }
        Message::CollapseLogsSidebar => {
            actions::collapse_logs_sidebar(app)
        }
        Message::CheckUpdate => {
            actions::check_update(app)
        }
        Message::CheckUpdateDone(msg) => {
            if !msg.trim().is_empty() {
                actions::push_log(app, format!("[info] {msg}"));
            }
            Command::none()
        }
        Message::DownloadUpdate => {
            actions::download_update(app)
        }
        Message::DownloadUpdateDone(msg) => {
            if !msg.trim().is_empty() {
                actions::push_log(app, format!("[info] {msg}"));
            }
            Command::none()
        }
        Message::ClearLogs => {
            app.logs.clear();
            app.logs_dirty = true;
            Command::none()
        }
        Message::CopyLogs => actions::copy_logs(app),
        Message::ExportLogs => actions::export_logs(app),
        Message::TrayMenu(id) => {
            let Some(tray) = &app.tray else {
                return Command::none();
            };
            if id == tray.toggle_id {
                app.minimized = !app.minimized;
                return iced::window::minimize(iced::window::Id::MAIN, app.minimized);
            }
            if id == tray.start_stop_id {
                return update(app, Message::StartStop);
            }
            if id == tray.check_update_id {
                return update(app, Message::CheckUpdate);
            }
            if id == tray.download_update_id {
                return update(app, Message::DownloadUpdate);
            }
            if id == tray.quit_id {
                return actions::quit(app);
            }
            Command::none()
        }
        Message::ToggleFollowLogs(v) => {
            app.follow_logs = v;
            app.touch_config();
            Command::none()
        }
    }
}
