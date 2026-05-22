mod config;
mod orchestrator;
mod tray;

use std::sync::{Arc, Mutex};
use tray_icon::menu::MenuEvent;

fn main() {
    // Initialize env_logger to support standard verbose log outputs.
    // If RUST_LOG environment variable is not defined, we default to "debug"
    // to satisfy the user's preference for verbose development logs.
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    log::info!("Starting TranscriberRUST background application...");

    // 1. Load configuration on startup
    let config = Arc::new(Mutex::new(config::Config::load()));
    log::debug!("Global configuration successfully locked in Arc Mutex.");

    // 2. Initialize coordinating channels
    let (event_sender, event_receiver) = std::sync::mpsc::channel::<orchestrator::AppEvent>();

    // 3. Spawn background event orchestrator thread
    let orchestrator_config = config.clone();
    std::thread::spawn(move || {
        orchestrator::run_orchestrator(event_receiver, orchestrator_config);
    });
    log::debug!("Orchestrator thread spawned.");

    // 4. Initialize native system tray icon and menus
    let tray_manager = tray::init_tray();

    // 5. Main event listener loop (polls menu item events from native Win32/macOS queues)
    let menu_channel = MenuEvent::receiver();

    log::info!("TranscriberRUST is now running in the system tray. Press Settings or Quit menu items.");

    loop {
        // Non-blocking poll for native menu item click events
        if let Ok(event) = menu_channel.try_recv() {
            log::debug!("Main thread captured menu event: {:?}", event);

            if event.id == tray_manager.settings_id {
                let _ = event_sender.send(orchestrator::AppEvent::OpenSettings);
            } else if event.id == tray_manager.quit_id {
                let _ = event_sender.send(orchestrator::AppEvent::Quit);
            }
        }

        // Rest the thread to prevent CPU spike (16ms equates to ~60Hz poll rate)
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
