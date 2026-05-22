# Milestone 2: System Tray & Main Event Loop Plan

Initialize the Windows background system tray menu, configure event propagation via message-passing channels (`mpsc`), implement the main background orchestrator thread, and integrate cross-platform event loops in `main.rs`.

---

## Settings

- **Testing:** Yes, add automated event tests and mock thread boundaries.
- **Logging:** Verbose (DEBUG logging enabled by default).
- **Docs:** Yes, update architectural diagrams and references upon completion.

---

## Tasks

### Phase 1: Native System Tray Interface

#### [NEW] [src/tray.rs](file:///c:/Users/tolib/Documents/GitHub/TranscriberRUST/src/tray.rs)
- **Task 1: Set up system tray icon and native menu items**
  - **Deliverable:** Implement `init_tray()` which registers a native `tray-icon` instance.
  - **Menu Structure:** Render standard menu items: `⚙ Settings` and `Quit`.
  - **Self-Contained Icon:** Implement `load_icon()` returning `tray_icon::Icon`. To avoid reliance on external assets, generate a simple 16x16 RGBA icon in-memory (e.g. standard circular indicator or solid colored block) to ensure 100% self-containment.
  - **Logging:** Add `log::debug!` detailing successful icon assembly and registration.
  - **Dependency:** None (Blocked by: none).

---

### Phase 2: Central Background Orchestration

#### [NEW] [src/orchestrator.rs](file:///c:/Users/tolib/Documents/GitHub/TranscriberRUST/src/orchestrator.rs)
- **Task 2: Implement coordinating event loops and state transitions**
  - **Deliverable:** Define the `AppEvent` event enum:
    - `HotkeyTriggered`
    - `OpenSettings`
    - `ConfigUpdated(Config)`
    - `Quit`
  - **Core Executor:** Implement `run_orchestrator(receiver, config)` processing events from a standard crossbeam/mpsc channel in a background thread.
  - **State Mocking:** Place clean debug prints for recording toggles to lay structural groundwork for CPAL capture (Milestone 4).
  - **Graceful Quit:** Implement safe system termination using `std::process::exit(0)` on `Quit`.
  - **Logging:** Emit `log::info!` on state transition completions and `log::debug!` for transient queue messages.
  - **Dependency:** Blocked by: Task 1.

---

### Phase 3: Main Bootstrap Coordination

#### [MODIFY] [src/main.rs](file:///c:/Users/tolib/Documents/GitHub/TranscriberRUST/src/main.rs)
- **Task 3: Integrate thread initialization and native tray event receivers**
  - **Deliverable:** Spawn the orchestrator in a dedicated thread.
  - **Native Loop listener:** Implement standard `tray_icon::TrayIconEvent` and `tray_icon::menu::MenuEvent` receiver pollers on the main thread (which is critical for platform window engines like Windows/macOS to dispatch tray interactions).
  - **Event Forwarding:** Translate native click events to standard `AppEvent` channels and forward them to the orchestrator thread.
  - **Logging:** Log thread spawn IDs and successful initial channel registrations.
  - **Dependency:** Blocked by: Task 2.

---

### Phase 4: Verification and Quality Assurance

- **Task 4: Implement automated event testing**
  - **Deliverable:** Add integration/unit tests validating thread-safe event dispatch and state changes under `src/orchestrator.rs`.
  - **Logging:** Ensure the test runner emits log events cleanly to support `is_test(true)` logging builders.
  - **Dependency:** Blocked by: Task 3.

- **Task 5: Final Clippy safety and compilation audits**
  - **Deliverable:** Compile warning-free and ensure all tests pass.
  - **Commands:** Run:
    ```powershell
    cargo test
    cargo clippy --all-targets --all-features -- -D warnings
    ```
  - **Dependency:** Blocked by: Task 4.

---

## Commit Plan

We will structure commits following the conventional commit specifications:

1. **Commit 1:** `feat(tray): implement native platform tray and in-memory RGBA icon generation` (After Task 1)
2. **Commit 2:** `feat(orchestrator): add background event loop thread and state handling` (After Task 2)
3. **Commit 3:** `feat(main): coordinate background thread spawning and tray menu event listeners` (After Task 3)
4. **Commit 4:** `test(event): add automated event channel validation tests and verify clippy` (After Task 4 and Task 5)
