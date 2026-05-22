# Plan: Milestone 1 - Project Skeleton & Configuration Layer

- **Branch:** `feature/project-skeleton-config`
- **Date:** 2026-05-22
- **Settings:**
  - Testing: Yes
  - Logging: Verbose (DEBUG)
  - Documentation: Yes

---

## Tasks

### Phase 1: Project Skeleton Setup

#### Task 1: Setup optimized Cargo.toml and project layout
- **Deliverable:** Fully configured `Cargo.toml` with size optimizations and required dependencies.
- **Files:** `Cargo.toml`
- **Details:**
  - Set up `[profile.release]` with size-reducing flags: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, and `strip = true`.
  - Add standard dependencies with correct features:
    - `egui` and `eframe` (v0.26+)
    - `tray-icon` (v0.14+)
    - `global-hotkey` (v0.5+)
    - `cpal` (v0.15+)
    - `hound` (v3.5+)
    - `ureq` (v2.9+ with default features)
    - `serde` (v1.0+ with `derive` feature)
    - `serde_json` (v1.0+)
    - `base64` (v0.22+)
    - `arboard` (v3.4+)
    - `enigo` (v0.2+)
    - `log` (v0.4+)
    - `env_logger` (v0.11+)
- **Logging:** Verify that compilation and dependency resolution succeed without warnings.
- **Dependencies:** None

---

### Phase 2: Configuration Engine

#### Task 2: Implement Config schema structures
- **Deliverable:** Define the thread-safe standard configuration struct with default values.
- **Files:** `src/config.rs`
- **Details:**
  - Create the `Config` structure matching the exact fields specified in `ai_prompt.md`:
    - `provider`: String ("openrouter" by default)
    - `openrouter_api_key`: String
    - `openrouter_model`: String ("google/gemini-3.1-flash-lite")
    - `openai_api_key`: String
    - `openai_model`: String ("whisper-1")
    - `openai_chat_model`: String ("gpt-4o-mini")
    - `groq_api_key`: String
    - `groq_model`: String ("whisper-large-v3")
    - `groq_chat_model`: String ("llama3-8b-8192")
    - `hotkey`: String ("ctrl+shift+space")
    - `insert_mode`: String ("typewriter")
    - `transcription_mode`: String ("clean")
    - `audio_duration_limit`: u32 (30)
    - `system_prompt`: String (Transcription instruction text)
  - Derive `Serialize`, `Deserialize`, `Clone`, and `Debug`.
  - Provide a `Default` implementation containing the default settings.
- **Logging:** Implement detailed `log::debug!` calls showing initialization parameters.
- **Dependencies:** Task 1

#### Task 3: Implement Config File I/O Persistence
- **Deliverable:** `load()` and `save()` operations targeting `~/.transcriber/config.json`.
- **Files:** `src/config.rs`
- **Details:**
  - Resolve the platform-agnostic home directory path (using the `home` crate or `std::env::var("USERPROFILE")` on Windows, or helper structures). Let's use `dirs` or `home` crate if added to `Cargo.toml`.
  - On `load()`:
    - Check if directory `~/.transcriber` exists. If not, create it.
    - Check if `~/.transcriber/config.json` exists.
    - If missing, write default `Config` to path, and return it.
    - If present, read and parse it. Handle missing fields gracefully.
  - On `save()`:
    - Serialize the struct with pretty formatting (`serde_json::to_string_pretty`).
    - Write safely to the path.
- **Logging:** Detailed logs:
  - `log::info!` on successful config file loading or creation.
  - `log::error!` on serialization or disk writing failure.
- **Dependencies:** Task 2

---

### Phase 3: Application Bootstrapper

#### Task 4: Setup main entry point and initialize logger
- **Deliverable:** `src/main.rs` that bootstraps the app, sets up logging, and validates config loading.
- **Files:** `src/main.rs`
- **Details:**
  - Create the `main` function.
  - Initialize the `env_logger` using `env_logger::init()` or `env_logger::Builder::from_default_env()`.
  - Load the configuration: `let config = Config::load();`.
  - Log details at startup: `log::info!("Starting Transcriber RUST...");`.
  - Log config details: `log::debug!("Loaded Config: {:?}", config);`.
  - Put a placeholder event loop or exit message to verify execution.
- **Logging:** 
  - Trace configuration properties on startup.
  - Log fatal errors if configuration loading crashes the program.
- **Dependencies:** Task 3

---

### Phase 4: Verification & Quality Gates

#### Task 5: Implement configuration unit tests
- **Deliverable:** Robust unit tests verifying configuration loading, saving, corruption recovery, and defaults.
- **Files:** `src/config.rs` (inline test module)
- **Details:**
  - Write test case: `test_default_config_fields` verifying initial values.
  - Write test case: `test_load_save_cycle` using a temporary directory or test filename to avoid overwriting user config.
  - Write test case: `test_corrupted_config_fallback` ensuring parsing failure falls back gracefully or handles missing fields.
- **Logging:** Print debug information during test execution.
- **Dependencies:** Task 4

#### Task 6: Validate build correctness and Clippy compliance
- **Deliverable:** Successful execution of tests and clean Clippy checks without warnings.
- **Files:** None (build check)
- **Details:**
  - Run `cargo check` and `cargo test`.
  - Run `cargo clippy --all-targets --all-features -- -D warnings`.
- **Logging:** Log Clippy outputs and test run summaries.
- **Dependencies:** Task 5

---

## Commit Plan

- **Commit 1:** `feat: set up optimised Cargo.toml and project layout` (After Task 1)
- **Commit 2:** `feat: implement config schema and load/save operations` (After Task 3)
- **Commit 3:** `feat: build main entry point, set up env_logger and configuration bootstrap` (After Task 4)
- **Commit 4:** `test: add robust unit tests for configuration serialization and fallbacks` (After Task 5)
- **Commit 5:** `chore: run clippy, verify size profile, and complete Milestone 1` (After Task 6)
