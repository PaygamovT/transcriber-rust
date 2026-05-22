# Architecture: Layered Service Architecture

## Overview
For the Rust rewrite of the **Transcriber** desktop utility, we adopt a horizontal **Layered Service Architecture**. This pattern isolates the user interface (`gui` and `tray`), the background control center (`orchestrator`), the system integration services (`audio`, `keyboard`, `hotkey`), and the data models (`config`). 

This keeps the codebase clean, ensures clear thread-safety boundaries, and avoids asynchronous runtime overhead, allowing the final release executable to compile to less than 3 MB with a ~3-7 MB active memory footprint.

## Decision Rationale
- **Project Type:** Desktop System Tray Background Utility.
- **Tech Stack:** Pure Rust + `egui`/`eframe` + `tray-icon` + `global-hotkey` + `cpal` + `hound` + `ureq` + `enigo` + `arboard`.
- **Key Factor:** Ultra-low memory usage, deterministic thread coordination, and quick execution start without heavy runtime engines (like `tokio`).

## Folder Structure
```
src/
├── main.rs              # Application entry point & thread initialization
├── config.rs            # Configuration structures and local JSON file persistence
├── tray.rs              # System tray icon setup & native platform menu actions
├── gui.rs               # egui Settings dark-themed dialog window view
├── hotkey.rs            # Global hotkey listening and event sender
├── audio.rs             # In-memory WAV recorder via cpal & hound
├── client.rs            # Synchronous AI transcription HTTP calls using ureq
├── keyboard.rs          # Unicode typewriter insertion (enigo) and clipboard setter
└── orchestrator.rs      # Coordination thread processing events & state transitions
```

## Dependency Rules

Dependencies strictly flow from top-level interface/coordination layers down to specialized low-level services.

```
                  ┌───────────────┐
                  │    main.rs    │
                  └───────┬───────┘
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │    tray.rs    │           │    gui.rs     │
    └───────┬───────┘           └───────┬───────┘
            │                           │
            └─────────────┬─────────────┘
                          ▼
               ┌────────────────────┐
               │  orchestrator.rs   │
               └──────────┬─────────┘
                          │
       ┌──────────────────┼──────────────────┐
       ▼                  ▼                  ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│   audio.rs   │   │  client.rs   │   │ keyboard.rs  │
└──────────────┘   └──────────────┘   └──────────────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          ▼
                   ┌──────────────┐
                   │  config.rs   │
                   └──────────────┘
```

- **Allowed Connections:**
  - `main.rs` initializes the background `orchestrator`, `tray`, and registers `hotkey`.
  - `orchestrator.rs` consumes services: `audio.rs` (recording), `client.rs` (APIs), and `keyboard.rs` (writing/typing).
  - All layers can read from `config.rs` structures.
- **Forbidden Connections:**
  - Services like `audio.rs` or `client.rs` must NEVER know about `gui.rs` or `tray.rs`.
  - `keyboard.rs` must not directly call `audio.rs`.

## Layer/Module Communication
The orchestrator communicates with the tray and global hotkey loop using standard **crossbeam/channel MPSC (Multi-Producer Single-Consumer)** channels.
- Hotkey triggers send an event to the orchestrator to start/stop recording.
- Tray menu selections send an event to open settings or terminate the app.

---

## Code Examples

### 1. The Orchestrator Event Loop Pattern
```rust
// src/orchestrator.rs
use crate::config::Config;
use std::sync::{Arc, Mutex};

pub enum AppEvent {
    HotkeyTriggered,
    OpenSettings,
    ConfigUpdated(Config),
    Quit,
}

pub fn run_orchestrator(
    receiver: std::sync::mpsc::Receiver<AppEvent>,
    config: Arc<Mutex<Config>>,
) {
    let mut recording_state = false;
    
    for event in receiver {
        match event {
            AppEvent::HotkeyTriggered => {
                if !recording_state {
                    // Start audio capture in memory
                    recording_state = true;
                    crate::audio::start_recording();
                } else {
                    // Stop capture, send WAV to client, and dispatch output
                    recording_state = false;
                    let wav_bytes = crate::audio::stop_recording().unwrap();
                    let current_config = config.lock().unwrap().clone();
                    
                    let text = crate::client::transcribe(&wav_bytes, &current_config).unwrap();
                    crate::keyboard::dispatch_output(&text, &current_config.insert_mode);
                }
            }
            AppEvent::OpenSettings => {
                // Open the egui settings frame
                crate::gui::open_settings_window(config.clone());
            }
            AppEvent::ConfigUpdated(new_config) => {
                let mut cfg = config.lock().unwrap();
                *cfg = new_config;
                // Re-register hotkeys if necessary
            }
            AppEvent::Quit => {
                std::process::exit(0);
            }
        }
    }
}
```

### 2. Configuration Operations (Thread-Safe Shared Config)
```rust
// src/config.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub provider: String,
    pub hotkey: String,
    pub insert_mode: String,
    // API keys, models, system prompt, etc.
}

impl Config {
    pub fn load() -> Self {
        // Load ~/.transcriber/config.json, create default if not found
        Default::default()
    }
    
    pub fn save(&self) -> Result<(), String> {
        // Write serialization to ~/.transcriber/config.json
        Ok(())
    }
}
```

## Anti-Patterns
- ❌ **No Async Runtime Bloat:** Do not pull in `tokio` or `async-std`. Keep HTTP network and audio recording operations synchronous and offloaded to simple operating system background worker threads.
- ❌ **No Ad-Hoc Thread State Sharing:** Do not share mutable application states across threads without structured `Arc<Mutex<T>>` or message-passing channels.
- ❌ **No Temporary Disk Files:** Do not write audio recording data to temporary WAV files. Keep all capture vectors and structures in-memory to respect user privacy and avoid disk I/O latency.
