use clap::Parser;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub dir: PathBuf,
    pub server_side_path_regex: Vec<String>,
    pub enable_server_side_file_size_limit: bool,
    pub max_server_side_file_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    CheckUpdate,
    DownloadUpdate,
    CliServer,
    #[cfg_attr(not(feature = "gui"), allow(dead_code))]
    Gui,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "f11esync", version, about = "F11eSync Rust server (Socket.IO compatible)")]
pub struct Cli {
    /// 端口号
    #[arg(short = 'p', long, default_value_t = 10080)]
    pub port: u16,

    /// 主机地址
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: IpAddr,

    /// 监听目录（等价于原 Bun 版本的 -d）
    #[arg(short = 'd', long)]
    pub dir: Option<PathBuf>,

    /// 检查更新（只输出结果后退出）
    #[arg(short = 'u', long)]
    pub update: bool,

    /// 下载新版本（默认下载到 Downloads；可用 F11ESYNC_DOWNLOAD_DIR 覆盖）
    #[arg(short = 'D', long)]
    pub download: bool,

    /// 强制无界面运行
    #[arg(long)]
    pub cli: bool,

    /// 服务端过滤规则（正则），可重复指定；默认会过滤 .DS_Store 与 __MACOSX
    #[arg(long = "path-regex")]
    pub path_regex: Vec<String>,

    /// 是否启用服务端文件大小限制（默认不启用；客户端也有自己的限制）
    #[arg(long)]
    pub enable_file_size_limit: bool,

    /// 服务端最大文件大小（字节）
    #[arg(long, default_value_t = 250 * 1024)]
    pub max_file_size: u64,
}

impl Cli {
    pub fn mode(&self) -> RunMode {
        if self.update {
            return RunMode::CheckUpdate;
        }
        if self.download {
            return RunMode::DownloadUpdate;
        }

        if self.cli {
            return RunMode::CliServer;
        }

        #[cfg(feature = "gui")]
        {
            RunMode::Gui
        }
        #[cfg(not(feature = "gui"))]
        {
            RunMode::CliServer
        }
    }

}
