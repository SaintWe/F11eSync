use crate::config::ServerConfig;
use crate::settings;
use crate::server::UiEvent;
use anyhow::Result;
use iced::multi_window;
use iced::widget::{container, text, text_editor};
use iced::{
    executor, time, Command, Element, Font, Length,
    Pixels, Settings, Size, Subscription, Theme,
};
use iced::{event, Event};
use std::time::Instant;
use tokio::sync::mpsc;

mod atoms;
mod actions;
mod atom_helper_config;
mod atom_helper_log;
mod commander;
mod data_officer;
mod diplomat;
mod molecules;
mod state;
mod styles;
mod view;
mod atom_api_doc;

use state::{BgServer, F11App, Message};

use styles::{
    AntCheckbox, AntDangerButton, AntPrimaryButton, AntSecondaryButton, AntTextButton, AntTextEditor, AntTextInput,
};

#[derive(Debug, Clone)]
pub struct GuiFlags {
    pub server: ServerConfig,
    pub ui: settings::UiConfig,
    pub needs_dir_pick: bool,
}

    fn preferred_font() -> Font {
        #[cfg(target_os = "macos")]
        {
            Font::with_name("PingFang SC")
        }
        #[cfg(target_os = "windows")]
        {
            Font::with_name("Microsoft YaHei UI")
        }
        #[cfg(target_os = "linux")]
        {
            Font::with_name("Noto Sans CJK SC")
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Font::DEFAULT
        }
    }

    pub fn run_gui(initial: GuiFlags) -> Result<()> {
        let mut settings = Settings::with_flags(initial);
        // 主窗口：启动前小窗；启动后再扩展出右侧日志边栏
        settings.window.size = Size::new(400.0, 480.0);
        settings.window.min_size = Some(settings.window.size);
        settings.window.resizable = false;
        settings.default_font = preferred_font();
        settings.default_text_size = Pixels(13.0);
        settings.window.icon = diplomat::window_icon();
        <F11App as multi_window::Application>::run(settings)?;
        Ok(())
    }

    impl multi_window::Application for F11App {
        type Executor = executor::Default;
        type Message = Message;
        type Theme = Theme;
        type Flags = GuiFlags;

        fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
            let (ui_tx, ui_rx) = mpsc::unbounded_channel::<UiEvent>();

            let regex = if flags.server.server_side_path_regex.is_empty() {
                text_editor::Content::with_text("\\.DS_Store$\n__MACOSX$\n")
            } else {
                text_editor::Content::with_text(&flags.server.server_side_path_regex.join("\n"))
            };

            let mut app = Self {
                host: flags.server.host.to_string(),
                port: flags.server.port.to_string(),
                dir: flags.server.dir,
                dir_input: String::new(),
                running: false,
                stopping: false,
                connected: false,
                last_error: String::new(),
                show_advanced: flags.ui.show_advanced,
                dark_mode: flags.ui.dark_mode,
                manual_dark_mode: flags.ui.dark_mode,
                follow_system_theme: flags.ui.follow_system_theme,
                server_side_regex: regex,
                enable_size_limit: flags.server.enable_server_side_file_size_limit,
                max_size: flags.server.max_server_side_file_size.to_string(),
                logs: Vec::new(),
                logs_dirty: false,
                follow_logs: flags.ui.follow_logs,
                show_logs_sidebar: false,
                config_dirty: false,
                config_last_change: Instant::now(),
                server: None,
                ui_rx,
                ui_tx,
                tray: diplomat::init_tray(),
                minimized: false,
                log_scroll_id: iced::widget::scrollable::Id::unique(),
                last_theme_check: Instant::now(),
            };
            if app.follow_system_theme {
                if let Some(dark) = Self::detect_dark_mode() {
                    app.dark_mode = dark;
                }
            }
            app.dir = std::fs::canonicalize(&app.dir).unwrap_or_else(|_| app.dir.clone());
            if flags.needs_dir_pick {
                app.dir_input.clear();
                app.last_error = "首次运行请先选择同步目录".to_string();
            } else {
                app.dir_input = app.dir.display().to_string();
            }
            actions::push_log(&mut app, "[info] F11eSync GUI 已启动".to_string());
            (app, Command::none())
        }

        fn title(&self, window: iced::window::Id) -> String {
            if window == iced::window::Id::MAIN {
                let s = if self.running { "运行中" } else { "已停止" };
                format!("F11eSync ({s})")
            } else {
                "F11eSync · 日志".to_string()
            }
        }

        fn theme(&self, _window: iced::window::Id) -> Self::Theme {
            atoms::ant_theme(self.dark_mode)
        }

        fn subscription(&self) -> Subscription<Self::Message> {
            Subscription::batch(vec![
                time::every(std::time::Duration::from_millis(200)).map(|_| Message::Tick),
                event::listen_with(|event, _status| match event {
                    Event::Window(id, iced::window::Event::CloseRequested) => {
                        Some(Message::CloseRequested(id))
                    }
                    _ => None,
                }),
            ])
        }

        fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
            commander::update(self, message)
        }

        fn view(&self, window: iced::window::Id) -> Element<'_, Self::Message> {
            if window == iced::window::Id::MAIN {
                view::main_window(self)
            } else {
                Element::from(container(text("")).width(Length::Fill).height(Length::Fill))
            }
        }
    }
