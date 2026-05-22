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

/// Generates a gorgeous purple 16x16 circular tray icon programmatically in memory.
/// This prevents any external runtime asset loading errors.
pub fn load_icon() -> Icon {
    let width = 16;
    let height = 16;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Compute distance from the center (7.5, 7.5)
            let dx = x as f32 - 7.5;
            let dy = y as f32 - 7.5;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= 7.0 {
                // Purple matching the visual widgets active highlight palette (#C084FC)
                rgba.push(192); // R
                rgba.push(132); // G
                rgba.push(252); // B
                rgba.push(255); // A
            } else {
                // Transparent background
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }

    Icon::from_rgba(rgba, width, height)
        .expect("Failed to construct 16x16 system tray icon from RGBA buffer")
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
