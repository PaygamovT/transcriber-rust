# Specification & System Prompt for Rebuilding Python Transcriber in Rust

You are Antigravity, a highly specialized Senior Rust Developer and Systems Architect. Your task is to rewrite a lightweight, zero-footprint desktop utility called **Transcriber** from Python (PyQt6/pynput) to **Rust**.

The ultimate goals of this rewrite are:
1. **Extremely Low Memory Footprint:** ~3-7 MB RAM usage in idle (idle Python uses ~40-60 MB).
2. **Tiny Standalone Binary:** Under 3-5 MB total executable size (.exe) when compiled for release and optimized.
3. **Mгновенный (Instant) Startup and Execution:** Zero interpreter initialization overhead.
4. **High Robustness:** Pure type-safe Rust with zero runtime dependency/missing packaging errors.

---

## 1. Project Overview & Features

**Transcriber** is a silent background application that sits in the Windows System Tray / macOS Menu Bar.
- **System Tray Operation:** The app starts completely hidden. No main window is shown on startup. It exposes a system tray icon with a context menu:
  - `Settings` (Opens the settings GUI window on-demand).
  - `Quit` (Gracefully exits the app).
- **Global Hotkey:** Listens system-wide for a configurable hotkey (default: `Ctrl+Shift+Space` on Windows, `Cmd+Shift+Space` on macOS).
- **Audio Capture:** When the hotkey is triggered, it records high-quality microphone input (16kHz, 1-channel mono, 16-bit PCM WAV) **purely in memory** (never writing temporary WAV files to the disk for maximum privacy and performance).
- **AI Transcription Backend:** Sends the in-memory audio (as Base64 for OpenRouter or multipart/form-data for OpenAI/Groq) to the configured API provider. It supports:
  - **OpenRouter** (calling `google/gemini-3.1-flash-lite` or similar model).
  - **OpenAI** (calling `whisper-1`).
  - **Groq** (calling `whisper-large-v3`).
- **Modes:**
  - **Clean:** Cleans verbal stutters ("uh", "um"), filler words, and repetitions.
  - **Translate:** Cleans stutters and translates the spoken words directly into fluent English.
- **Output Handlers:**
  - **Clipboard:** Copies the transcribed text to the system clipboard.
  - **Typewriter:** Types the transcribed text character-by-character at the current active cursor position (supporting Russian, Uzbek, and English Unicode symbols perfectly).

---

## 2. Technical Stack & Recommended Crate Architecture

To maintain the absolute smallest memory footprint and binary size, avoid dragging in heavy webview libraries (like Tauri) unless absolutely necessary. Instead, use a lightweight native Rust GUI library or lightweight immediate-mode GUI:

1. **System Tray & GUI Window:** 
   - **`tray-icon`** crate (cross-platform native system tray icon and context menus).
   - **`egui`** + **`eframe`** (incredibly lightweight immediate-mode GUI for the settings window, which compiles to a tiny native window, utilizing minimal RAM and under 1.5MB executable footprint) OR **`slint`** (declarative native UI).
2. **Global Hotkey:**
   - **`global-hotkey`** crate (cross-platform global keyboard shortcuts listener).
3. **Audio Capture:**
   - **`cpal`** crate (Cross-Platform Audio Library) for native microphone access.
   - **`hound`** crate (to generate WAV headers and PCM structures in memory via `std::io::Cursor`).
4. **HTTP Client & Serialization:**
   - **`ureq`** (minimalist synchronous HTTP client that compiles to ~100KB, avoiding the heavy `tokio` dependency that `reqwest` pulls in) OR **`reqwest`** (if async is preferred).
   - **`serde`** + **`serde_json`** (for config and API payload serialization).
   - **`base64`** (for encoding audio).
5. **Clipboard & Emulated Keyboard Typing:**
   - **`arboard`** crate (robust clipboard setter).
   - **`enigo`** crate (for typing emulation at cursor, ensuring proper Unicode character insertion).

---

## 3. Configuration System

Configuration must be stored locally as a JSON file at `~/.transcriber/config.json`.

### Default Configuration Fields:
```json
{
  "provider": "openrouter",
  "openrouter_api_key": "",
  "openrouter_model": "google/gemini-3.1-flash-lite",
  "openai_api_key": "",
  "openai_model": "whisper-1",
  "openai_chat_model": "gpt-4o-mini",
  "groq_api_key": "",
  "groq_model": "whisper-large-v3",
  "groq_chat_model": "llama3-8b-8192",
  "hotkey": "ctrl+shift+space",
  "insert_mode": "typewriter",
  "transcription_mode": "clean",
  "audio_duration_limit": 30,
  "system_prompt": "You are a pure audio transcription tool. Your ONLY task is to transcribe exactly what is spoken in the audio. Do NOT generate new text, do NOT hallucinate, do NOT complete sentences, and do NOT add any extra information. If you hear nothing or only noise, return an empty string. The audio may contain speech in Russian, English, or Uzbek. Return ONLY the transcribed text in its original language, exactly as spoken. No explanations, no translations, no prefixes."
}
```

---

## 4. UI/UX Specification: Settings Window Design Guide

Recreate the high-fidelity, premium dark-themed PyQt6 settings window exactly. If you are using **`egui`**, you must customize the `egui::Visuals` and fonts to match these precise specs.

### Color Palette & Theme Specs:
- **Font Family:** Modern typography like `'Outfit'`, `'Inter'`, or `'Segoe UI'` (sans-serif).
- **Backgrounds:**
  - Outer Dialog Window: `#0D0F12` (Ultra-dark deep slate/black).
  - Inner Settings Card/GroupBox: `#1A1C23` (Dark charcoal grey).
  - Input Fields / ComboBox / Textbox: `#15171C` (Very dark grey).
- **Borders & Grid:** `#232631` (Subtle dark outline).
- **Accents & Highlights:** `#C084FC` (Vibrant pastel purple/lavender).
- **Typography Colors:**
  - Header / Active Titles: `#FFFFFF` (Pure white).
  - Form Fields Labels: `#C4C4CC` (Soft light grey).
  - Placeholders / Inactive elements: `#8C8E98` / `#A8A8B3` (Muted medium grey).
- **Save Button Gradient:** Linear gradient from `#A855F7` to `#7C3AED` (Deep rich purple), with hover states fading to `#C084FC` -> `#8B5CF6`.
- **Cancel Button Style:** Transparent background, subtle border `#232631`, text color `#A8A8B3`.

### Window Layout & Dimensions:
- **Size:** Minimum `540x480` pixels. Non-resizable or with clean responsive layouts.
- **Margins & Spacing:** Outer padding `24px`. Spacing between fields `16px`. Spacing between sections `20px`.
- **Header:**
  - Display a gear icon `⚙️` in `#C084FC` (size 20px) followed by the bold title **"Transcriber Settings"** (size 20px) in white.
- **Tab Navigation Bar:**
  - Two buttons sitting side-by-side representing tabs:
    1. **`❖  API Settings`**
    2. **`🎙️  Recording & Playback`**
  - Clicking a tab dynamically flips the content panel. The active tab has a solid `#C084FC` bottom border (2px) and purple text. The inactive tab has muted `#8C8E98` text.

### Tab Contents & Interactive Behaviors:

#### Tab 1: API Settings
1. **API Provider:** Dropdown/Combobox with options: `OpenRouter`, `OpenAI`, `Groq`.
   - **Dynamic Provider Caching (Crucial!):** When the user switches the active provider:
     - Automatically cache the currently typed API Key and Model in memory.
     - Dynamically change the **API Key** label (e.g. `API Key OpenRouter` or `API Key OpenAI`).
     - Swap out the **Transcription Model** presets dynamically:
       - **OpenRouter Presets:** `google/gemini-3.1-flash-lite`, `openai/whisper-large-v3`, `meta-llama/llama-3-70b-instruct`.
       - **OpenAI Presets:** `whisper-1`.
       - **Groq Presets:** `whisper-large-v3`, `whisper-large-v3-turbo`, `distil-whisper-large-v3-en`.
2. **API Key Input:**
   - Must mask characters by default (password/bullet style).
   - Must contain an inline visibility toggle button on the far right (eye icon `👁️` or `🙈`) that toggles masking.
3. **Transcription Model:** Editable dropdown (combo-box) that allows selecting from the presets above OR typing in a custom model name.

#### Tab 2: Recording & Playback
1. **Global Hotkey Input:** Text field displaying current hotkey sequence, with placeholder `ctrl+shift+space`.
2. **Text Insertion Mode:** Dropdown containing:
   - `Печатать на курсоре (Typewriter)` -> value: `typewriter`
   - `Копировать в буфер (Clipboard)` -> value: `clipboard`
3. **Transcription Mode:** Dropdown containing:
   - `Очистка от повторов и пауз (Clean)` -> value: `clean`
   - `Очистка + Перевод на английский (Translate)` -> value: `translate`

#### Bottom Controls:
- Row containing two buttons aligned to the right:
  - **Cancel (Отмена):** Discards changes, closes settings.
  - **Save (Сохранить):** Writes all fields (including the values for all API providers cached in memory) to `config.json`, triggers system-wide configuration reload, and closes settings.

---

## 5. Architectural Implementation Guidelines

Follow this phase-by-phase implementation sequence:

### Phase 1: Configuration & File I/O
Define the configuration structures with `serde(default)` values. Implement `load()` and `save()` helper methods targeting `~/.transcriber/config.json`. Ensure the application creates the `.transcriber` directory dynamically if it does not exist.

### Phase 2: Native System Tray Icon
Initialize the platform tray icon using the `tray-icon` crate. Establish a simple main loop where the application runs purely in the background (no main window opened on launch). Hook up a menu handler for `Settings` (which opens the eframe/egui dialog window on-demand) and `Quit` (which terminates the loop). Ensure closing the settings window completely frees its memory resources (do not just hide it; destroy the window context).

### Phase 3: Global Hotkey Listener
Wire up the `global-hotkey` crate to monitor system keyboard events. Parse the configuration string (e.g. `ctrl+shift+space`) and register it. Upon interception, send an event or message to the main state thread.

### Phase 4: Audio Capture & WAV Generation
Use `cpal` to open the default microphone input stream.
- When hotkey is pressed first time: Start recording. Store incoming f32 samples dynamically in a vector.
- When hotkey is pressed again (or safety limit is hit): Stop recording and close stream.
- Normalization & WAV assembly: Normalize f32 samples to i16 range (-32768 to 32767). Use `hound` to write a fully compliant mono 16kHz 16-bit WAV header and payload **entirely in a memory buffer** (`Vec<u8>`).

### Phase 5: API Transcription Client
Implement the transcription client using `ureq` (or `reqwest`).
- **OpenRouter:** Encode WAV bytes in Base64. Check if the model is a general-purpose chat/multimodal model (e.g. Gemini). If it is, construct a `/v1/chat/completions` request. Include the system prompt and a message containing the `input_audio` block.
- **OpenAI / Groq:** Send the raw WAV bytes using `multipart/form-data` to `/v1/audio/transcriptions`. If mode is `clean` or `translate`, send the resulting text to `/v1/chat/completions` with the respective system prompt to clean or translate the text.

### Phase 6: Output Processor
- **Clipboard Mode:** Pass the resulting text directly into the `arboard` clipboard writer.
- **Typewriter Mode:** Emulate key events using `enigo`. Ensure you handle unicode characters properly. On Windows, use native virtual key codes or Unicode injection hooks to type Russian/Uzbek Cyrillic symbols without messing up characters due to active layout differences.

---

## 6. Binary Optimization Checklist (Must Apply on Cargo.toml!)

To compress the final Rust executable size to the absolute minimum, ensure you configure these compilation flags in `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Enable Link-Time Optimization
codegen-units = 1    # Reduce parallel code generation units for better optimization
panic = "abort"      # Remove heavy stack unwinding panic landing pads
strip = true         # Strip symbols and debug info automatically
```

With this specification, proceed to implement the full-featured, zero-footprint Rust version of **Transcriber**. Maintain clean code boundaries, separate the presenter/orchestrator from the low-level OS services, and provide descriptive logging.
