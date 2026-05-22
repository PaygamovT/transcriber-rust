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

/// Actions routed back to the main thread for coordination.
#[derive(Clone, Debug, PartialEq)]
pub enum MainThreadAction {
    ReRegisterHotkey,
}

/// Primary background event processing engine.
/// Monitors event messages, toggles recording status, schedules API client queries,
/// and delegates typewriter input simulation.
pub fn run_orchestrator(
    receiver: std::sync::mpsc::Receiver<AppEvent>,
    config: Arc<Mutex<Config>>,
    main_sender: std::sync::mpsc::Sender<MainThreadAction>,
) {
    log::info!("Background event orchestrator thread successfully started.");
    let mut recording_state = false;

    // Track active CPAL streaming context inside the orchestrator state
    let mut active_stream: Option<cpal::Stream> = None;
    let mut audio_buffer: Option<Arc<Mutex<Vec<f32>>>> = None;
    let mut native_sample_rate: u32 = 0;

    for event in receiver {
        log::debug!("Orchestrator received event: {:?}", event);

        match event {
            AppEvent::HotkeyTriggered => {
                recording_state = !recording_state;
                if recording_state {
                    log::info!("🎤 Audio recording started [State: Active]");
                    match crate::audio::start_recording() {
                        Ok(context) => {
                            active_stream = Some(context.stream);
                            audio_buffer = Some(context.buffer);
                            native_sample_rate = context.sample_rate;
                        }
                        Err(e) => {
                            log::error!("Failed to start dynamic microphone capture stream: {}", e);
                            recording_state = false; // Reset state since starting failed
                        }
                    }
                } else {
                    log::info!("⏹ Audio recording stopped [State: Idle]");

                    // Gracefully stop recording by taking/dropping the active CPAL stream
                    let _stream = active_stream.take();

                    if let Some(buffer_ref) = audio_buffer.take() {
                        // Lock the buffer to extract raw captured audio frames
                        let raw_samples = {
                            let guard = buffer_ref
                                .lock()
                                .expect("Failed to lock recorded sample buffer");
                            guard.clone()
                        };

                        log::info!(
                            "Captured {} raw audio frames from default microphone input.",
                            raw_samples.len()
                        );

                        if !raw_samples.is_empty() {
                            // 1. Resample from hardware native rate to standard 16000 Hz mono
                            let resampled =
                                crate::audio::resample(&raw_samples, native_sample_rate, 16000);

                            // 2. Perform absolute peak amplitude normalization and scale to standard signed i16 PCM
                            let pcm_samples = crate::audio::normalize_and_convert(&resampled);

                            // 3. Compile samples to standard WAV byte buffer in RAM
                            match crate::audio::encode_wav_in_memory(&pcm_samples) {
                                Ok(wav_bytes) => {
                                    log::info!(
                                        "Successfully finalized in-memory WAV compilation! Captured {} bytes.",
                                        wav_bytes.len()
                                    );
                                    // TODO: Send compiled WAV bytes to configured API Client (Milestones 5 & 6)
                                }
                                Err(e) => {
                                    log::error!("Failed to compile in-memory WAV bytes: {}", e);
                                }
                            }
                        } else {
                            log::warn!("No audio samples were captured. Skipping compilation.");
                        }
                    }

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
                let old_hotkey = {
                    let active_config = config.lock().expect("Failed to lock config mutex");
                    active_config.hotkey.clone()
                };

                let hotkey_changed = old_hotkey != new_config.hotkey;

                let mut active_config = config.lock().expect("Failed to lock config mutex");
                *active_config = *new_config;

                if hotkey_changed {
                    log::info!(
                        "Hotkey combination changed from '{}' to '{}'. Requesting re-registration.",
                        old_hotkey,
                        active_config.hotkey
                    );
                    if let Err(e) = main_sender.send(MainThreadAction::ReRegisterHotkey) {
                        log::error!("Failed to notify main thread to re-register hotkey: {:?}", e);
                    }
                }
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
        let (main_sender, main_receiver) = mpsc::channel::<MainThreadAction>();
        let config = Arc::new(Mutex::new(Config::default()));

        // 2. Spawn orchestrator in a background worker thread
        let thread_config = config.clone();
        let handle = std::thread::spawn(move || {
            run_orchestrator(receiver, thread_config, main_sender);
        });

        // 3. Dispatch dynamic update event using struct initializer (modifying hotkey to trigger ReRegister)
        let new_config = Config {
            provider: "groq".to_string(),
            transcription_mode: "translate".to_string(),
            hotkey: "ctrl+shift+a".to_string(), // different hotkey
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
        assert_eq!(active.hotkey, "ctrl+shift+a");

        // 6. Assert that MainThreadAction::ReRegisterHotkey was sent
        let main_action = main_receiver.try_recv();
        assert_eq!(main_action, Ok(MainThreadAction::ReRegisterHotkey));

        // 7. Explicitly drop the sender to close the loop and join the worker thread
        drop(sender);
        let _ = handle.join();
    }
}
