use crate::config::{Cli, ServerConfig};
use anyhow::{Context, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub follow_system_theme: bool,
    pub dark_mode: bool,
    pub show_advanced: bool,
    pub follow_logs: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            follow_system_theme: true,
            dark_mode: false,
            show_advanced: false,
            follow_logs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfigFile {
    pub host: IpAddr,
    pub port: u16,
    pub dir: PathBuf,
    pub server_side_path_regex: Vec<String>,
    pub enable_file_size_limit: bool,
    pub max_file_size: u64,
}

impl Default for ServerConfigFile {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".parse().unwrap(),
            port: 10080,
            dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("skin"),
            server_side_path_regex: vec![r"\.DS_Store$".to_string(), r"__MACOSX$".to_string()],
            enable_file_size_limit: false,
            max_file_size: 250 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub schema_version: u32,
    pub server: ServerConfigFile,
    pub ui: UiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            server: ServerConfigFile::default(),
            ui: UiConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub server: ServerConfig,
    #[cfg(feature = "gui")]
    pub ui: UiConfig,
}

fn is_cli(matches: &ArgMatches, id: &str) -> bool {
    matches.value_source(id) == Some(ValueSource::CommandLine)
}

fn ensure_abs_dir(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(path)
}

fn canonicalize_best_effort(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| ensure_abs_dir(path))
}

pub fn config_file_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").unwrap_or_default();
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("F11eSync")
            .join("config.yaml");
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("F11eSync").join("config.yaml");
        }
        let home = std::env::var_os("USERPROFILE").unwrap_or_default();
        return PathBuf::from(home)
            .join("AppData")
            .join("Roaming")
            .join("F11eSync")
            .join("config.yaml");
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        return base.join("f11esync").join("config.yaml");
    }
}

pub fn load() -> Result<Option<AppConfig>> {
    let path = config_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(&path).with_context(|| format!("读取配置失败: {}", path.display()))?;
    let cfg: AppConfig =
        serde_yaml::from_slice(&bytes).with_context(|| format!("解析配置失败: {}", path.display()))?;
    Ok(Some(cfg))
}

#[cfg(feature = "gui")]
pub fn save(cfg: &AppConfig) -> Result<()> {
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
    }

    let yaml = serde_yaml::to_string(cfg).context("序列化配置失败")?;
    let tmp = path.with_extension("yaml.tmp");
    std::fs::write(&tmp, yaml.as_bytes())
        .with_context(|| format!("写入临时配置失败: {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| format!("替换配置失败: {}", path.display()))?;
    Ok(())
}

pub fn merge(cli: &Cli, matches: &ArgMatches, file: Option<AppConfig>) -> EffectiveConfig {
    let mut base = file.unwrap_or_default();

    // Ensure directory is absolute even when loaded from disk.
    base.server.dir = canonicalize_best_effort(&base.server.dir);

    if is_cli(matches, "host") {
        base.server.host = cli.host;
    }
    if is_cli(matches, "port") {
        base.server.port = cli.port;
    }
    if is_cli(matches, "dir") {
        if let Some(dir) = &cli.dir {
            base.server.dir = canonicalize_best_effort(dir);
        }
    }
    if is_cli(matches, "path_regex") && !cli.path_regex.is_empty() {
        base.server.server_side_path_regex = cli.path_regex.clone();
    }
    if is_cli(matches, "enable_file_size_limit") {
        base.server.enable_file_size_limit = cli.enable_file_size_limit;
    }
    if is_cli(matches, "max_file_size") {
        base.server.max_file_size = cli.max_file_size;
    }

    let server = ServerConfig {
        host: base.server.host,
        port: base.server.port,
        dir: base.server.dir.clone(),
        server_side_path_regex: base.server.server_side_path_regex.clone(),
        enable_server_side_file_size_limit: base.server.enable_file_size_limit,
        max_server_side_file_size: base.server.max_file_size,
    };

    EffectiveConfig {
        server,
        #[cfg(feature = "gui")]
        ui: base.ui,
    }
}
