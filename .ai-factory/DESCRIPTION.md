# Project: TranscriberRUST

## Overview
A lightweight, zero-footprint Windows system tray desktop utility written in pure Rust that listens for a global hotkey, captures high-quality microphone audio entirely in memory, transcribes/translates it via AI backend APIs (OpenRouter, OpenAI, Groq), and outputs the resulting text directly to the system clipboard or types it character-by-character at the active cursor position.

This is a complete high-performance, low-memory rewrite of a legacy Python (PyQt6/pynput) utility.

## Core Features
- **Silent Background Operation:** Starts hidden in the Windows System Tray.
- **On-Demand GUI Settings:** Beautiful dark-themed egui/eframe settings dialog opened from the tray menu.
- **Global Hotkey:** Listens for `Ctrl+Shift+Space` (or custom hotkey) system-wide.
- **In-Memory Audio Capture:** Records 16kHz mono 16-bit PCM WAV using `cpal` and `hound` entirely in RAM.
- **AI Transcription Backends:**
  - **OpenRouter:** Supports audio transcription using chat/multimodal API inputs (e.g. Gemini 1.5/3.1 Flash).
  - **OpenAI:** Uses standard Whisper (`whisper-1`) endpoint.
  - **Groq:** Uses rapid Whisper (`whisper-large-v3`) endpoint.
- **Processing Modes:**
  - **Clean:** Removes verbal stutters ("uh", "um"), filler words, and repetitions.
  - **Translate:** Cleans verbal stutters and translates the audio directly into English.
- **Output Handlers:**
  - **Clipboard:** Copies transcribed text to the clipboard using `arboard`.
  - **Typewriter:** Types out Unicode text character-by-character using `enigo` (perfect support for Russian, Uzbek, and English).

## Tech Stack
- **Language:** Pure Rust
- **GUI Framework:** `egui` + `eframe` (ultra-lightweight immediate-mode GUI for settings window)
- **System Tray:** `tray-icon`
- **Global Hotkey:** `global-hotkey`
- **Audio Capture & WAV:** `cpal` (audio capture) + `hound` (in-memory WAV generation)
- **HTTP Client:** `ureq` (minimalist synchronous HTTP client for tiny executable size and no heavy tokio dependency)
- **Serialization:** `serde` + `serde_json`
- **Base64 Encoding:** `base64`
- **Clipboard:** `arboard`
- **Typing Emulation:** `enigo`
- **Database:** None (configurations stored locally in `~/.transcriber/config.json`)
- **ORM:** None

## Architecture
See `.ai-factory/ARCHITECTURE.md` for detailed architecture guidelines.
Pattern: Layered Service Architecture

## Non-Functional Requirements
- **Logging:** Configurable via `env_logger` or similar simple logger.
- **Error Handling:** Graceful, non-crashing handling of missing microphone permissions, network errors, or API key validation.
- **Security:** API keys and credentials saved locally in user directory with restricted file permissions (`config.json`).
