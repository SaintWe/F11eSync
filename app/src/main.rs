mod app;
mod config;
mod proto;
mod server;
mod settings;
mod update;
mod watcher;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use config::{Cli, RunMode};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn".into()),
        )
        .init();

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    match cli.mode() {
        RunMode::CheckUpdate => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(update::check_update(cli.update_silent))?;
            Ok(())
        }
        RunMode::DownloadUpdate => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(update::download_update())?;
            Ok(())
        }
        RunMode::CliServer => {
            let file = settings::load().unwrap_or(None);
            let effective = settings::merge(&cli, &matches, file);
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server::run_headless(effective.server))?;
            Ok(())
        }
        RunMode::Gui => {
            #[cfg(feature = "gui")]
            {
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
            #[cfg(not(feature = "gui"))]
            {
                anyhow::bail!("此二进制未启用 gui feature（请用 `--features gui` 编译）")
            }
        }
    }
}
