[Back to README](../README.md) · [Architecture →](architecture.md)

# Getting Started Guide

This guide will walk you through setting up your Windows development environment, installing the required GCC toolchains, building the TranscriberRUST executable, and running build validation steps.

---

## Windows Prerequisites

TranscriberRUST depends on native system calls for audio capture (`cpal`) and typing emulation (`enigo`). On Windows, this requires the **GNU Compiler Collection (GCC)** linker instead of standard MSVC tools to maintain a zero-dependency standalone output binary.

### 1. Install Scoop
If you do not have Scoop installed, open a PowerShell terminal and run:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

### 2. Install GCC and Rustup GNU Toolchain
Install the GCC linker suite and configure Rust to target the GNU ABI:
```powershell
# Install GCC and rustup
scoop install gcc rustup-gnu

# Add the stable GNU toolchain as default
rustup default stable-x86_64-pc-windows-gnu
```

---

## Compiling TranscriberRUST

Since the linker requires GCC, you must ensure the compiler binaries are available in your shell's active path during the build process.

### Step-by-Step Build Commands

1. **Set GCC Path (PowerShell):**
   ```powershell
   $env:Path += ";C:\Users\tolib\scoop\apps\gcc\current\bin"
   ```
2. **Compile in Debug Mode (for local development logs):**
   ```powershell
   cargo build
   ```
3. **Compile Optimized Release Binary:**
   ```powershell
   cargo build --release
   ```

---

## Verifying the Build

To verify that the application bootstraps and links correctly:

### 1. Run Unit Tests
Validate serialization routines, env logger behaviors, and recovery routines:
```powershell
cargo test
```
All tests should finish with `ok`.

### 2. Check the Executable Size
The release profile is aggressive. Confirm that `target/release/transcriber.exe` is **under 1 MB**:
```powershell
Get-Item .\target\release\transcriber.exe | Select-Object Name, @{Name="Size (KB)"; Expression={$_.Length / 1KB}}
```
Expect an output size of approximately **930 KB to 950 KB** (due to link-time optimization and symbol stripping).

### 3. Run with Debug Logging
Execute the bootstrapper and check for the debug loaded config events:
```powershell
# Run with active debug logging
$env:RUST_LOG="debug"
.\target\debug\transcriber.exe
```
Verify the output displays your active config directory location:
```text
DEBUG [transcriber::config] Loading configuration from path: "C:\\Users\\<user>\\.transcriber\\config.json"
INFO  [transcriber::config] Configuration loaded successfully.
```

---

## See Also

- [System Architecture](architecture.md) — Understanding the thread models and event channels.
- [Configuration Reference](configuration.md) — Editing credentials and system transcription prompts.
