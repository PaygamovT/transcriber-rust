#![windows_subsystem = "windows"]

mod audio;
mod client;
mod config;
mod gui;
mod hotkey;
mod orchestrator;
mod output;
mod tray;

use std::sync::{Arc, Mutex};
use tray_icon::menu::MenuEvent;
use global_hotkey::GlobalHotKeyEvent;

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
    let (main_sender, main_receiver) = std::sync::mpsc::channel::<orchestrator::MainThreadAction>();

    // 3. Spawn background event orchestrator thread
    let orchestrator_config = config.clone();
    std::thread::spawn(move || {
        orchestrator::run_orchestrator(event_receiver, orchestrator_config, main_sender);
    });
    log::debug!("Orchestrator thread spawned.");

    // 4. Initialize native system tray icon and menus
    let tray_manager = tray::init_tray();

    // 5. Initialize global hotkey listener and register key triggers
    let mut hotkey_state = hotkey::HotkeyState::new();
    let initial_hotkey = {
        let config_guard = config.lock().expect("Failed to lock config on startup");
        config_guard.hotkey.clone()
    };
    if let Err(e) = hotkey_state.update_hotkey(&initial_hotkey) {
        log::error!(
            "Failed to register initial global hotkey '{}': {}",
            initial_hotkey,
            e
        );
    }

    // 6. Main event listener loop (polls events from native queues)
    let menu_channel = MenuEvent::receiver();
    let hotkey_channel = GlobalHotKeyEvent::receiver();

    log::info!(
        "TranscriberRUST is now running. Active hotkey: '{}'.",
        initial_hotkey
    );

    loop {
        #[cfg(target_os = "windows")]
        pump_win32_messages();

        // A. Non-blocking poll for native menu item click events
        if let Ok(event) = menu_channel.try_recv() {
            log::debug!("Main thread captured menu event: {:?}", event);

            if event.id == tray_manager.settings_id {
                let _ = event_sender.send(orchestrator::AppEvent::OpenSettings);
            } else if event.id == tray_manager.quit_id {
                let _ = event_sender.send(orchestrator::AppEvent::Quit);
            }
        }

        // B. Non-blocking poll for global hotkey trigger events
        if let Ok(event) = hotkey_channel.try_recv() {
            log::debug!("Main thread captured hotkey event: {:?}", event);

            if let Some(ref current_hk) = hotkey_state.current_hotkey {
                if event.id == current_hk.id() && event.state == global_hotkey::HotKeyState::Pressed {
                    log::info!("Global hotkey pressed! Forwarding to orchestrator.");
                    let _ = event_sender.send(orchestrator::AppEvent::HotkeyTriggered);
                }
            }
        }

        // C. Non-blocking poll for commands from the background orchestrator
        if let Ok(action) = main_receiver.try_recv() {
            log::debug!("Main thread received command action: {:?}", action);

            match action {
                orchestrator::MainThreadAction::ReRegisterHotkey => {
                    let updated_hotkey = {
                        let config_guard = config.lock().expect("Failed to lock config");
                        config_guard.hotkey.clone()
                    };
                    log::info!("Re-registering global hotkey to: '{}'", updated_hotkey);
                    if let Err(e) = hotkey_state.update_hotkey(&updated_hotkey) {
                        log::error!("Failed to update global hotkey: {}", e);
                    }
                }
                orchestrator::MainThreadAction::OpenSettingsWindow => {
                    let gui_config = config.clone();
                    let gui_sender = event_sender.clone();
                    log::info!("Opening settings window on the main thread...");
                    crate::gui::open_settings_window(gui_config, gui_sender);
                    log::info!("Settings window closed. Re-initializing global hotkey state...");

                    // Re-initialize hotkey_state to re-register on the refreshed Win32 message queue
                    hotkey_state = hotkey::HotkeyState::new();
                    let current_hk = {
                        let config_guard = config.lock().expect("Failed to lock config after settings close");
                        config_guard.hotkey.clone()
                    };
                    if let Err(e) = hotkey_state.update_hotkey(&current_hk) {
                        log::error!("Failed to re-register hotkey after settings close: {}", e);
                    }
                }
            }
        }

        // Rest the thread to prevent CPU spike (16ms equates to ~60Hz poll rate)
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

#[cfg(target_os = "windows")]
fn pump_win32_messages() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };
    use std::mem::zeroed;
    unsafe {
        let mut msg: MSG = zeroed();
        while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
