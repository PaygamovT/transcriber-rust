# TranscriberRUST

> Ultra-lightweight Windows system tray background utility written in pure Rust that listens for a global hotkey, captures microphone audio entirely in memory, transcribes/translates it via AI backend APIs, and types the result or copies it to the clipboard.

TranscriberRUST is a high-performance, zero-footprint rewrite of a legacy Python utility. By utilizing low-level Rust libraries and a synchronous HTTP architecture, it achieves a tiny standalone binary footprint under **5 MB** (fully self-contained including all GUI and graphics assets) and keeps RAM usage under **7 MB** while idle.

---

## Key Features

- 🎧 **In-Memory Audio Capture:** Captures high-quality 16kHz mono audio in RAM via `cpal` + `hound`, avoiding slow disk writes and ensuring complete privacy.
- ⌨️ **Unicode Typewriter Emulation:** Automatically emulates keypresses via `enigo` to type transcription text character-by-character at the active cursor position. Supports English, Russian, and Uzbek out of the box.
- ⚡ **AI Transcription Backends:** Built-in connectors for **OpenRouter** (default, using Gemini Flash models), **OpenAI** (Whisper-1), and **Groq** (rapid Whisper-large-v3).
- ⚙️ **Dark-Themed Settings GUI:** Minimal settings dialog created using `egui` and `eframe` which triggers directly from the platform system tray.
- 🛡️ **Robust Local Configuration:** Persists configurations securely in `~/.transcriber/config.json` with corruption-safe automatic recovery.

---

## Quick Start

### 1. Prerequisites (Windows)
Ensure you have the GNU toolchain and GCC linker installed. We recommend installing them via [Scoop](https://scoop.sh):
```powershell
scoop install gcc rustup-gnu
rustup default stable-x86_64-pc-windows-gnu
```

### 2. Build and Run
Clone the repository and compile the highly optimized standalone release binary:
```powershell
# Add GCC to the path for compile-time linking
$env:Path += ";C:\Users\tolib\scoop\apps\gcc\current\bin"

# Build optimized binary
cargo build --release

# Run the bootstrapper
./target/release/transcriber-rust.exe
```

---

## Configuration Example

The settings file is located at `~/.transcriber/config.json`. Below is an example configuration:
```json
{
  "provider": "openrouter",
  "openrouter_api_key": "your-api-key-here",
  "openrouter_model": "google/gemini-3.1-flash-lite",
  "hotkey": "ctrl+shift+space",
  "insert_mode": "typewriter",
  "transcription_mode": "clean",
  "audio_duration_limit": 30
}
```

---

## Documentation

For deep dives into project structure, APIs, and setup instructions, explore the topic-specific guides:

| Topic | Description |
|---|---|
| [Getting Started Guide](docs/getting-started.md) | Full environment setup, GCC toolchain details, and compiler troubleshooting. |
| [System Architecture](docs/architecture.md) | High-level layered module layout, background event loop model, and thread guidelines. |
| [Configuration Reference](docs/configuration.md) | Comprehensive parameter breakdown, default values, and fallback JSON restoration details. |

---

## License

This project is licensed under the [MIT License](LICENSE).
