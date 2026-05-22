# Project Roadmap

> A lightweight, zero-footprint desktop utility that sits in the system tray, records microphone audio entirely in memory on hotkey trigger, transcribes/translates using OpenRouter/OpenAI/Groq, and outputs the result via typewriter emulation or clipboard.

## Milestones

- [x] **Milestone 1: Project Skeleton & Configuration Layer** — Configure `Cargo.toml` optimized for size, establish directories, and implement `~/.transcriber/config.json` loader/saver with all default provider caching fields.
- [x] **Milestone 2: System Tray & Main Event Loop** — Set up platform system tray icon using `tray-icon` with `Settings` and `Quit` items, running in a headless background loop and managed via thread communication channels.
- [x] **Milestone 3: Global Hotkey Listener** — Integrate the `global-hotkey` crate to capture user-defined key triggers (like `Ctrl+Shift+Space`) globally and relay state events to the background thread.
- [x] **Milestone 4: In-Memory Audio Capture (CPAL)** — Connect to the default input microphone, capture mono 16kHz audio in real-time, scale/normalize samples, and pack WAV headers using `hound` entirely in RAM.
- [ ] **Milestone 5: OpenRouter Multimodal Client** — Build `ureq` HTTP helper to encode WAV bytes into Base64 and run multimodal completions using the OpenRouter Gemini API backend.
- [ ] **Milestone 6: OpenAI & Groq Whisper Client** — Implement synchronous HTTP multipart/form-data builder to send WAV streams to standard Whisper APIs.
- [ ] **Milestone 7: AI Processing Modes** — Set up LLM text refinement pipelines to automatically strip speech stutters (Clean Mode) or translate transcriptions into English (Translate Mode).
- [ ] **Milestone 8: Emulated Typing & Clipboard Output** — Hook up `arboard` for clipboard injection and `enigo` for layout-safe Unicode typewriter insertion at the current cursor position.
- [ ] **Milestone 9: Dark-Themed Settings GUI** — Design a premium, tabbed `egui`/`eframe` dark dialog window with dynamic provider caches, password masked text boxes, and instant window context cleanup on close.
- [ ] **Milestone 10: Size & RAM Footprint Audits** — Implement Cargo compilation profiles (LTO, strip, abort panic), audit RAM usage to guarantee under 7MB idle memory, and package final binaries.

## Completed

| Milestone | Date |
|-----------|------|
| Milestone 1: Project Skeleton & Configuration Layer | 2026-05-22 |
| Milestone 2: System Tray & Main Event Loop | 2026-05-22 |
| Milestone 3: Global Hotkey Listener | 2026-05-22 |
| Milestone 4: In-Memory Audio Capture (CPAL) | 2026-05-22 |
