#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(feature = "gui"))]
compile_error!("f11esync-gui 需要启用 gui feature（例如 `cargo build --bin f11esync-gui --features gui`）");

#[path = "../app/mod.rs"]
mod app;
#[path = "../config.rs"]
mod config;
#[path = "../proto.rs"]
mod proto;
#[path = "../server/mod.rs"]
mod server;
#[path = "../settings.rs"]
mod settings;
#[path = "../update.rs"]
mod update;
#[path = "../watcher.rs"]
mod watcher;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use config::Cli;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn".into()),
        )
        .init();

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    let file = settings::load().unwrap_or(None);
    let needs_dir_pick = file.is_none();
    let effective = settings::merge(&cli, &matches, file);
    app::run_gui(app::GuiFlags {
        server: effective.server,
        ui: effective.ui,
        needs_dir_pick,
    })?;
    Ok(())
}
