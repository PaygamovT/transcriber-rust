use tray_icon::{
    menu::{Menu, MenuId, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

#[allow(dead_code)]
pub struct TrayManager {
    pub tray_icon: TrayIcon,
    pub settings_id: MenuId,
    pub quit_id: MenuId,
}

/// Loads the gorgeous custom application icon from assets.
/// This embeds the PNG directly into the binary at compile time.
pub fn load_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let decoded = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Png)
        .expect("Failed to load icon from memory")
        .into_rgba8();
    let (width, height) = decoded.dimensions();
    let rgba = decoded.into_raw();
    Icon::from_rgba(rgba, width, height)
        .expect("Failed to construct system tray icon")
}

/// Initializes the native Windows system tray icon and attaches action menu triggers.
pub fn init_tray() -> TrayManager {
    log::debug!("Initializing native platform system tray icon and context menus.");

    let tray_menu = Menu::new();
    let settings_item = MenuItem::new("⚙ Settings", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    let settings_id = settings_item.id().clone();
    let quit_id = quit_item.id().clone();

    tray_menu
        .append_items(&[&settings_item, &quit_item])
        .expect("Failed to append system tray menu items");

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("TranscriberRUST")
        .with_icon(load_icon())
        .build()
        .expect("Failed to construct native platform TrayIcon");

    log::info!("System tray icon registered successfully.");

    TrayManager {
        tray_icon,
        settings_id,
        quit_id,
    }
}
