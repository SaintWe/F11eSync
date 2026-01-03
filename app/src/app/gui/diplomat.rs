use std::fs;
use std::path::PathBuf;

pub fn pick_folder(current: &PathBuf) -> Option<PathBuf> {
    rfd::FileDialog::new().set_directory(current).pick_folder()
}

pub fn save_log_file(current: &PathBuf) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory(current)
        .set_file_name("f11esync.log")
        .save_file()
}

pub fn ensure_dir_and_canonicalize(path: &PathBuf) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(path)?;
    Ok(std::fs::canonicalize(path).unwrap_or_else(|_| path.clone()))
}

pub fn save_settings(app_cfg: &crate::settings::AppConfig) -> anyhow::Result<()> {
    crate::settings::save(app_cfg)?;
    Ok(())
}

pub fn find_asset(name: &str) -> Option<PathBuf> {
    let name = name.trim_start_matches(['/', '\\']);

    let mut roots: Vec<PathBuf> = Vec::new();

    if let Ok(dir) = std::env::var("F11ESYNC_ASSETS_DIR") {
        roots.push(PathBuf::from(dir));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("assets"));

            #[cfg(target_os = "macos")]
            {
                roots.push(dir.join("../Resources/assets"));
            }
        }
    }

    roots.push(PathBuf::from("assets"));
    roots.push(PathBuf::from("app/assets"));

    roots
        .into_iter()
        .map(|root| root.join(name))
        .find(|p| p.exists())
}

pub fn load_png_rgba(path: &PathBuf) -> Option<(Vec<u8>, u32, u32)> {
    let bytes = fs::read(path).ok()?;
    load_png_rgba_from_bytes(&bytes)
}

pub fn load_png_rgba_from_bytes(bytes: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((rgba.into_raw(), w, h))
}

fn embedded_icon_png() -> &'static [u8] {
    include_bytes!("../../../assets/icon.png")
}

fn embedded_tray_png_2x() -> &'static [u8] {
    include_bytes!("../../../assets/trayTemplate@2x.png")
}

fn embedded_tray_png_1x() -> &'static [u8] {
    include_bytes!("../../../assets/trayTemplate.png")
}

pub fn tray_icon() -> Option<tray_icon::Icon> {
    let from_file = find_asset("trayTemplate@2x.png")
        .and_then(|p| load_png_rgba(&p))
        .or_else(|| find_asset("trayTemplate.png").and_then(|p| load_png_rgba(&p)));

    let (rgba, w, h) = from_file
        .or_else(|| load_png_rgba_from_bytes(embedded_tray_png_2x()))
        .or_else(|| load_png_rgba_from_bytes(embedded_tray_png_1x()))?;
    tray_icon::Icon::from_rgba(rgba, w, h).ok()
}

pub fn window_icon() -> Option<iced::window::Icon> {
    let from_file = find_asset("icon.png").and_then(|p| load_png_rgba(&p));
    let (rgba, w, h) = from_file.or_else(|| load_png_rgba_from_bytes(embedded_icon_png()))?;
    iced::window::icon::from_rgba(rgba, w, h).ok()
}

pub struct TrayHandle {
    pub _tray: tray_icon::TrayIcon,
    pub toggle_id: String,
    pub start_stop_id: String,
    pub check_update_id: String,
    pub download_update_id: String,
    pub quit_id: String,
}

pub fn init_tray() -> Option<TrayHandle> {
    use tray_icon::menu::{Menu, MenuItem};
    use tray_icon::TrayIconBuilder;

    let icon = tray_icon()?;

    let menu = Menu::new();
    let toggle = MenuItem::new("最小化/恢复", true, None);
    let start_stop = MenuItem::new("启动/停止", true, None);
    let check_update = MenuItem::new("检查更新", true, None);
    let download_update = MenuItem::new("下载更新", true, None);
    let quit = MenuItem::new("退出", true, None);

    let _ = menu.append(&toggle);
    let _ = menu.append(&start_stop);
    let _ = menu.append(&check_update);
    let _ = menu.append(&download_update);
    let _ = menu.append(&quit);

    let tray = TrayIconBuilder::new()
        .with_tooltip("F11eSync")
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .with_icon_as_template(cfg!(target_os = "macos"))
        .build()
        .ok()?;

    Some(TrayHandle {
        _tray: tray,
        toggle_id: toggle.id().0.clone(),
        start_stop_id: start_stop.id().0.clone(),
        check_update_id: check_update.id().0.clone(),
        download_update_id: download_update.id().0.clone(),
        quit_id: quit.id().0.clone(),
    })
}

pub fn try_recv_tray_event_id() -> Option<String> {
    use tray_icon::menu::MenuEvent;
    let Ok(ev) = MenuEvent::receiver().try_recv() else {
        return None;
    };
    Some(ev.id.0)
}
