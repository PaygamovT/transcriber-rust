---
name: egui-tray-application
description: >-
  Architectural guidelines and code patterns for building a lightweight desktop
  background application in Rust using egui/eframe, tray-icon, global-hotkey,
  cpal, hound, ureq, enigo, and arboard. Use when designing, refactoring,
  or implementing the background loop, tray context menu, settings window, global hotkeys,
  or audio/typing operations in Transcriber.
argument-hint: "[action]"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read Write Bash(*)
metadata:
  author: AI Factory
  version: "1.0"
  category: desktop-development
---

# Egui Tray Application Architecture & Guidelines

This skill defines the architectural guidelines, state machine design, and code patterns for building a production-ready, ultra-low memory background desktop utility in Rust.

---

## 1. System Tray & Window Lifecycle Manager

To achieve ~3-7 MB idle RAM, the application must run headless in the background, only initializing the Settings GUI on-demand. When the user closes the settings window, the GUI context should be fully destroyed to reclaim memory, rather than simply hiding the window.

### Threading & Main Event Loop
Use a dedicated cross-platform main thread for the system tray loop, global hotkeys, and coordination. Since `egui`/`eframe` requires taking over the main thread on some platforms (like macOS), the tray execution must coordinate with `eframe::run_native`.

> [!CRITICAL]
> **Windows/Winit Main Thread Execution:**
> `eframe::run_native` initialization (and any winit event loop creation) MUST run strictly on the **main thread**. Attempting to invoke `run_native` from a background `std::thread` will cause an instant panic on Windows ("Initializing the event loop outside of the main thread is a significant cross-platform compatibility hazard"). 
> Ensure the settings window is launched synchronously on the main thread (coordinating via a channel if triggered from system-tray or other threads), using `NativeOptions` configured with `run_and_return: true` to return execution control back to the tray event loop when the window is closed.

### Tray Menu Creation (`tray-icon` Crate)
Create a native tray icon and a menu containing `Settings` and `Quit`.

```rust
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};

pub fn init_tray() -> tray_icon::TrayIcon {
    let tray_menu = Menu::new();
    let settings_item = MenuItem::new("⚙ Settings", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    
    tray_menu.append_items(&[&settings_item, &quit_item]).unwrap();
    
    TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Transcriber")
        .with_icon(load_icon()) // Returns tray_icon::Icon
        .build()
        .unwrap()
}
```

---

## 2. Dynamic Egui Dialog Window & Caching

The Settings window is a dark-themed GUI based on `egui`/`eframe`. It must support:
1. **Dynamic Caching:** Swapping providers (OpenRouter, OpenAI, Groq) should preserve the inputs for other providers in memory until the "Save" button is clicked.
2. **Password Masking:** API key text field should be masked by default with a toggleable eye button.
3. **Tabbed Navigation:** Left/Top tabs to switch between API Settings and Recording/Playback.

### Visual Styling Specs (`egui::Visuals`)
```rust
pub fn apply_custom_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();
    
    // Custom Color Palette
    visuals.window_fill = egui::Color32::from_hex("#0D0F12").unwrap(); // Deep dark slate
    visuals.faint_bg_color = egui::Color32::from_hex("#1A1C23").unwrap(); // Inner GroupBox
    visuals.widgets.inactive.bg_fill = egui::Color32::from_hex("#15171C").unwrap(); // Input Fields
    visuals.widgets.hovered.bg_fill = egui::Color32::from_hex("#232631").unwrap();
    visuals.widgets.active.bg_fill = egui::Color32::from_hex("#C084FC").unwrap(); // Accent Highlights
    
    style.visuals = visuals;
    ctx.set_style(style);
}
```

### On-Demand Window Spawning & Destruction
Do not keep the `eframe` window hidden. Spawn it via `eframe::run_native` when the settings event is received, and ensure closing it terminates that specific window runner cleanly while returning control to the tray loop.
*Tip:* To avoid exiting the entire process when the `eframe` window closes, configure `eframe::NativeOptions` with `run_and_return: true` and custom close behavior.

---

## 3. Global Hotkey Registration (`global-hotkey` Crate)

Parse configuration string like `ctrl+shift+space` to register keyboard shortcuts:

```rust
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use std::str::FromStr;

pub fn register_hotkey(manager: &GlobalHotKeyManager, hotkey_str: &str) -> Result<HotKey, String> {
    let parts: Vec<&str> = hotkey_str.to_lowercase().split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut key_code = None;
    
    for part in parts {
        match part.trim() {
            "ctrl" | "control" => modifiers.insert(Modifiers::CONTROL),
            "shift" => modifiers.insert(Modifiers::SHIFT),
            "alt" => modifiers.insert(Modifiers::ALT),
            "meta" | "cmd" | "super" => modifiers.insert(Modifiers::SUPER),
            other => {
                key_code = Some(parse_key_code(other)?);
            }
        }
    }
    
    let key = key_code.ok_or("No key specified in hotkey string")?;
    let hotkey = HotKey::new(Some(modifiers), key);
    manager.register(hotkey).map_err(|e| e.to_string())?;
    Ok(hotkey)
}
```

---

## 4. In-Memory Audio Capture & WAV Normalization (`cpal` & `hound` Crates)

Audio capture must be executed in RAM without saving intermediate `.wav` files on disk.

### Capture Loop Checklist:
- Open default input device using `cpal::default_host()`.
- Use a standard configuration: **16000 Hz, 1 Channel (mono), 16-bit PCM WAV**.
- Capture floats (`f32`) from input stream and append to an atomic or thread-safe shared `Arc<Mutex<Vec<f32>>>`.
- **Normalization:** Find the peak absolute amplitude `max_val` of the captured `f32` vector. Scale all samples so the absolute peak matches `0.9` of full scale. Map to `i16` bounds `[-32768, 32767]`.
- **WAV Assembly:** Assemble the WAV bytes in-memory using `std::io::Cursor` and `hound`.

```rust
use hound::{WavSpec, WavWriter};
use std::io::Cursor;

pub fn encode_wav_in_memory(samples: &[i16]) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    {
        let mut writer = WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;
        for &sample in samples {
            writer.write_sample(sample).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
    }
    
    Ok(cursor.into_inner())
}
```

---

## 5. Synchronous HTTP Transcriptions Client (`ureq` Crate)

Avoid importing standard `tokio` / `reqwest` async frameworks which bloat binary size.
Perform API requests using synchronous `ureq`.

### 1. OpenRouter (Multimodal API)
Multimodal APIs accept standard base64 formatted inputs. Convert WAV bytes to base64, then submit via `/v1/chat/completions`:

```rust
let base64_audio = base64::engine::general_purpose::STANDARD.encode(&wav_bytes);
let payload = serde_json::json!({
    "model": "google/gemini-3.1-flash-lite",
    "messages": [
        { "role": "system", "content": system_prompt },
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": "Transcribe this audio exactly."
                },
                {
                    "type": "input_audio",
                    "input_audio": {
                        "data": base64_audio,
                        "format": "wav"
                    }
                }
            ]
        }
    ]
});

let response: serde_json::Value = ureq::post("https://openrouter.ai/api/v1/chat/completions")
    .set("Authorization", &format!("Bearer {}", api_key))
    .send_json(payload)
    .map_err(|e| e.to_string())?
    .into_json()
    .map_err(|e| e.to_string())?;
```

### 2. OpenAI / Groq (`/v1/audio/transcriptions` Multipart)
OpenAI and Groq whisper endpoints require `multipart/form-data`.
Since `ureq` does not have a built-in multipart writer, manually construct the boundary headers and payload, or use a tiny helper function:

```rust
let boundary = "----RustTranscriberBoundary123456";
let mut body = Vec::new();

// Add Model Part
body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
body.extend_from_slice(b"Content-Disposition: form-data; name=\"model\"\r\n\r\n");
body.extend_from_slice(format!("{}\r\n", model).as_bytes());

// Add File Part
body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"audio.wav\"\r\n");
body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
body.extend_from_slice(&wav_bytes);
body.extend_from_slice(b"\r\n");

// Close Boundary
body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

let response: serde_json::Value = ureq::post("https://api.openai.com/v1/audio/transcriptions")
    .set("Authorization", &format!("Bearer {}", api_key))
    .set("Content-Type", &format!("multipart/form-data; boundary={}", boundary))
    .send_bytes(&body)
    .map_err(|e| e.to_string())?
    .into_json()
    .map_err(|e| e.to_string())?;
```

---

## 6. Output Processors (`enigo` & `arboard` Crates)

### Typewriter Emulation (Unicode Insertion Hook)
Emulate key presses character-by-character. Ensure proper Unicode compatibility for Cyrillic/Uzbek Unicode characters without depending on the user's active keyboard layout on Windows.
- Standard typing emulation (`key_click` / `key_down`) is subject to layout mapping and can type garbage characters.
- **WARNING:** Direct keyboard text emulation via `enigo.text()` is highly unreliable for multi-byte Unicode/non-ASCII strings (Cyrillic, Uzbek, etc.) on Windows, often corrupting or truncating the string after the first few words.
- **SOLUTION:** Use the robust **Clipboard-Paste fallback pattern** for reliable multi-language typing insertion: save the current clipboard content, put the target text into the clipboard using `arboard`, emulate `Ctrl+V` key presses, sleep briefly to let the OS process the event, and then restore the original clipboard content.

```rust
use arboard::Clipboard;
use enigo::{Enigo, Settings, Keyboard, Direction, Key};
use std::thread::sleep;
use std::time::Duration;

pub fn emulate_typing(text: &str) {
    // 1. Capture and backup current clipboard content
    let mut clipboard = Clipboard::new().ok();
    let old_content = clipboard.as_mut().and_then(|cb| cb.get_text().ok());
    
    // 2. Load the target text into the clipboard
    if let Some(ref mut cb) = clipboard {
        if cb.set_text(text).is_ok() {
            // 3. Emulate Ctrl+V keyboard shortcut
            if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                let _ = enigo.key(Key::Control, Direction::Press);
                let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                let _ = enigo.key(Key::Control, Direction::Release);
                
                // Allow a brief yield for the OS buffer to complete pasting
                sleep(Duration::from_millis(150));
            }
        }
    }
    
    // 4. Restore original clipboard content
    if let (Some(mut cb), Some(old)) = (clipboard, old_content) {
        let _ = cb.set_text(&old);
    }
}
```

### Clipboard Handler
Write strings directly to clipboard:
```rust
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}
```
