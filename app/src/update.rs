use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};

const REPO_NAME: &str = "SaintWe/F11eSync";
const GITHUB_API_URL: &str = "https://api.github.com/repos/SaintWe/F11eSync/releases/latest";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn compare_versions(a: &str, b: &str) -> i32 {
    let a_parts: Vec<u32> = a.split('.').filter_map(|x| x.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|x| x.parse().ok()).collect();
    for i in 0..a_parts.len().max(b_parts.len()) {
        let ap = *a_parts.get(i).unwrap_or(&0);
        let bp = *b_parts.get(i).unwrap_or(&0);
        if ap > bp {
            return 1;
        }
        if ap < bp {
            return -1;
        }
    }
    0
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_download_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("F11ESYNC_DOWNLOAD_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOMEPATH"))
            .map(PathBuf::from)?;
        return Some(home.join("Downloads"));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        return Some(home.join("Downloads"));
    }
}

fn platform_zip_name() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        if !cfg!(feature = "gui") {
            return Some("f11esync-rust-windows-x64.zip");
        }
        return Some("f11esync-gui-windows-x64.zip");
    }
    if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        if cfg!(feature = "gui") {
            return Some("f11esync-gui-darwin-x64.zip");
        }
        return Some("f11esync-darwin-x64.zip");
    }
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        if cfg!(feature = "gui") {
            return Some("f11esync-gui-darwin-arm64.zip");
        }
        return Some("f11esync-darwin-arm64.zip");
    }
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        if !cfg!(feature = "gui") {
            return Some("f11esync-rust-linux-x64.zip");
        }
        return Some("f11esync-linux-x64.zip");
    }
    if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        if !cfg!(feature = "gui") {
            return Some("f11esync-rust-linux-arm64.zip");
        }
        return Some("f11esync-linux-arm64.zip");
    }
    None
}

async fn fetch_latest_release() -> Result<Release> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("f11esync")
        .build()?;
    let res = client
        .get(GITHUB_API_URL)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("请求 GitHub API 失败")?;
    if !res.status().is_success() {
        anyhow::bail!("GitHub API 返回错误: {}", res.status());
    }
    Ok(res.json::<Release>().await?)
}

#[derive(Debug, Clone)]
pub enum DownloadUpdateResult {
    Skipped { local: String, remote: String },
    Downloaded { remote: String, path: PathBuf },
}

pub async fn check_update_message(silent: bool) -> Result<String> {
    let release = fetch_latest_release().await?;
    let remote = release.tag_name.trim_start_matches('v').to_string();
    let local = current_version();

    if compare_versions(&remote, &local) > 0 {
        Ok(format!(
            "发现新版本: v{remote} (当前: v{local})，下载: https://github.com/{REPO_NAME}/releases/latest"
        ))
    } else if silent {
        Ok(String::new())
    } else {
        Ok(format!("当前已是最新版本: v{local}"))
    }
}

pub async fn check_update(silent: bool) -> Result<()> {
    let msg = match check_update_message(silent).await {
        Ok(message) => message,
        Err(err) => {
            if !silent {
                warn!("检查更新失败: {err:#}");
            }
            return Ok(());
        }
    };
    if !msg.is_empty() {
        info!("{}", msg);
    }
    Ok(())
}

pub async fn download_update() -> Result<DownloadUpdateResult> {
    let zip_name = platform_zip_name().context("不支持的平台/架构")?;
    let release = fetch_latest_release().await?;
    let remote = release.tag_name.trim_start_matches('v').to_string();
    let local = current_version();

    if compare_versions(&remote, &local) <= 0 {
        info!("当前版本 v{local} 已是最新或高于远程版本 v{remote}，无需下载");
        return Ok(DownloadUpdateResult::Skipped { local, remote });
    }

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == zip_name)
        .with_context(|| format!("未找到当前平台的下载文件: {zip_name}"))?;

    info!("发现新版本: v{remote}，开始下载...");
    info!("下载地址: {}", asset.browser_download_url);

    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent("f11esync")
        .build()?;
    let res = client.get(&asset.browser_download_url).send().await?;
    if !res.status().is_success() {
        anyhow::bail!("下载失败: HTTP {}", res.status());
    }
    let bytes = res.bytes().await?;

    let out_dir = default_download_dir()
        .or_else(|| std::env::current_dir().ok())
        .context("无法确定下载保存目录")?;
    let out_path = out_dir.join(zip_name);
    let _ = tokio::fs::create_dir_all(&out_dir).await;

    tokio::fs::write(&out_path, &bytes).await?;

    info!("下载完成: {}", out_path.display());
    info!("新版本: v{remote}");
    info!("请解压后替换当前程序");
    Ok(DownloadUpdateResult::Downloaded {
        remote,
        path: out_path,
    })
}

pub async fn download_update_message() -> Result<String> {
    match download_update().await? {
        DownloadUpdateResult::Skipped { local, remote } => {
            Ok(format!("当前已是最新版本: v{local}（远端: v{remote}），无需下载"))
        }
        DownloadUpdateResult::Downloaded { remote, path } => Ok(format!(
            "下载完成: {}（新版本: v{remote}）",
            path.display()
        )),
    }
}
