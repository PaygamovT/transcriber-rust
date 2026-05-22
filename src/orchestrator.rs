use crate::config::Config;
use std::sync::{Arc, Mutex};

/// Events processed by the central coordination thread.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    HotkeyTriggered,
    OpenSettings,
    ConfigUpdated(Box<Config>),
    Quit,
}

/// Primary background event processing engine.
/// Monitors event messages, toggles recording status, schedules API client queries,
/// and delegates typewriter input simulation.
pub fn run_orchestrator(
    receiver: std::sync::mpsc::Receiver<AppEvent>,
    config: Arc<Mutex<Config>>,
) {
    log::info!("Background event orchestrator thread successfully started.");
    let mut recording_state = false;

    for event in receiver {
        log::debug!("Orchestrator received event: {:?}", event);

        match event {
            AppEvent::HotkeyTriggered => {
                recording_state = !recording_state;
                if recording_state {
                    log::info!("🎤 Audio recording started [State: Active]");
                    // TODO: Connect CPAL audio capture stream (Milestone 4)
                } else {
                    log::info!("⏹ Audio recording stopped [State: Idle]");
                    // TODO: Finalize WAV compilation, perform AI API query, and emulate typing
                    let current_config = config.lock().expect("Failed to lock config mutex");
                    log::debug!(
                        "Active transcription settings: Provider={}, Mode={}",
                        current_config.provider,
                        current_config.transcription_mode
                    );
                }
            }
            AppEvent::OpenSettings => {
                log::info!("🖥 Requested to launch GUI settings panel");
                // TODO: Spin up on-demand egui/eframe window context (Milestone 9)
            }
            AppEvent::ConfigUpdated(new_config) => {
                log::info!("⚙ Configuration changes applied dynamically");
                let mut active_config = config.lock().expect("Failed to lock config mutex");
                *active_config = *new_config;
                // TODO: Re-register global system hooks if the trigger combination changed (Milestone 3)
            }
            AppEvent::Quit => {
                log::info!("🚪 Received quit instruction. Terminating background threads gracefully.");
                std::process::exit(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_orchestrator_event_propagation() {
        let _ = env_logger::builder().is_test(true).try_init();

        // 1. Setup channel and configurations
        let (sender, receiver) = mpsc::channel::<AppEvent>();
        let config = Arc::new(Mutex::new(Config::default()));

        // 2. Spawn orchestrator in a background worker thread
        let thread_config = config.clone();
        let handle = std::thread::spawn(move || {
            run_orchestrator(receiver, thread_config);
        });

        // 3. Dispatch dynamic update event using struct initializer
        let new_config = Config {
            provider: "groq".to_string(),
            transcription_mode: "translate".to_string(),
            ..Config::default()
        };

        sender
            .send(AppEvent::ConfigUpdated(Box::new(new_config)))
            .unwrap();

        // Dispatch a toggle recording signal to test active loops
        sender.send(AppEvent::HotkeyTriggered).unwrap();
        sender.send(AppEvent::HotkeyTriggered).unwrap();

        // 4. Wait brief interval to let processing cycle execute
        std::thread::sleep(Duration::from_millis(50));

        // 5. Assert modifications applied inside the ArcMutex correctly
        let active = config.lock().unwrap();
        assert_eq!(active.provider, "groq");
        assert_eq!(active.transcription_mode, "translate");

        // 6. Explicitly drop the sender to close the loop and join the worker thread
        drop(sender);
        let _ = handle.join();
    }
}
