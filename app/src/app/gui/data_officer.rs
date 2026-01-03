use crate::config::ServerConfig;
use crate::settings;
use iced::widget::text_editor;
use std::path::PathBuf;

use super::atom_helper_config;

pub fn build_server_config(
    host: &str,
    port: &str,
    dir: &PathBuf,
    server_side_regex: &text_editor::Content,
    enable_size_limit: bool,
    max_size: &str,
) -> Option<ServerConfig> {
    let (host, port) = atom_helper_config::parse_host_port(host, port)?;
    let server_side_path_regex = atom_helper_config::parse_server_side_path_regex(server_side_regex);
    let max_server_side_file_size = atom_helper_config::parse_max_server_side_file_size(max_size);

    Some(ServerConfig {
        host,
        port,
        dir: dir.clone(),
        server_side_path_regex,
        enable_server_side_file_size_limit: enable_size_limit,
        max_server_side_file_size,
    })
}

pub fn build_app_config(
    server_cfg: &ServerConfig,
    server_side_regex: &text_editor::Content,
    follow_system_theme: bool,
    manual_dark_mode: bool,
    show_advanced: bool,
    follow_logs: bool,
) -> settings::AppConfig {
    let server_side_path_regex = atom_helper_config::parse_server_side_path_regex(server_side_regex);
    settings::AppConfig {
        schema_version: 1,
        server: settings::ServerConfigFile {
            host: server_cfg.host,
            port: server_cfg.port,
            dir: server_cfg.dir.clone(),
            server_side_path_regex,
            enable_file_size_limit: server_cfg.enable_server_side_file_size_limit,
            max_file_size: server_cfg.max_server_side_file_size,
        },
        ui: settings::UiConfig {
            follow_system_theme,
            dark_mode: manual_dark_mode,
            show_advanced,
            follow_logs,
        },
    }
}

