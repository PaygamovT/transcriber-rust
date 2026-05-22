use global_hotkey::{
    hotkey::HotKey,
    GlobalHotKeyManager,
};
use std::str::FromStr;

/// State manager for the active global hotkey.
/// Keeps track of the `GlobalHotKeyManager` instance and the currently registered hotkey,
/// allowing clean unregistration and registration when hotkeys change.
pub struct HotkeyState {
    pub manager: GlobalHotKeyManager,
    pub current_hotkey: Option<HotKey>,
}

impl HotkeyState {
    /// Creates a new `HotkeyState` by initializing the global hotkey manager.
    pub fn new() -> Self {
        log::debug!("Initializing HotkeyState and native GlobalHotKeyManager.");
        let manager = GlobalHotKeyManager::new().expect("Failed to initialize GlobalHotKeyManager");
        Self {
            manager,
            current_hotkey: None,
        }
    }

    /// Dynamically updates the registered global hotkey. Unregisters the old hotkey
    /// and registers the new one parsed from the string representation.
    pub fn update_hotkey(&mut self, hotkey_str: &str) -> Result<(), String> {
        // 1. Unregister the old hotkey if it exists
        if let Some(ref hk) = self.current_hotkey {
            log::debug!("Unregistering existing hotkey (ID: {})", hk.id());
            if let Err(e) = self.manager.unregister(*hk) {
                log::warn!("Failed to unregister hotkey (ID: {}): {:?}", hk.id(), e);
            }
        }

        // 2. Parse and register the new hotkey
        match parse_and_register(&self.manager, hotkey_str) {
            Ok(hk) => {
                self.current_hotkey = Some(hk);
                Ok(())
            }
            Err(e) => {
                self.current_hotkey = None;
                Err(e)
            }
        }
    }
}

impl Default for HotkeyState {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a hotkey string (e.g. "ctrl+shift+space") and registers it with the given manager.
/// Returns the parsed HotKey on success, or a descriptive error message on failure.
pub fn parse_and_register(
    manager: &GlobalHotKeyManager,
    hotkey_str: &str,
) -> Result<HotKey, String> {
    log::debug!("Attempting to parse hotkey string: '{}'", hotkey_str);

    // Parse the hotkey combination using the FromStr implementation from the global-hotkey crate.
    // E.g., "ctrl+shift+space" -> Modifiers::CONTROL | Modifiers::SHIFT and Code::Space
    let hotkey = HotKey::from_str(hotkey_str)
        .map_err(|e| format!("Failed to parse hotkey string '{}': {:?}", hotkey_str, e))?;

    log::debug!(
        "Registering hotkey with modifiers: {:?}, key: {:?} (ID: {})",
        hotkey.mods,
        hotkey.key,
        hotkey.id()
    );

    manager
        .register(hotkey)
        .map_err(|e| format!("Failed to register hotkey in global manager: {:?}", e))?;

    log::info!("Successfully registered hotkey '{}' (ID: {})", hotkey_str, hotkey.id());
    Ok(hotkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_hotkeys() {
        let _ = env_logger::builder().is_test(true).try_init();

        // Verify that standard hotkey formats parse successfully
        let valid_cases = vec![
            "ctrl+shift+space",
            "Ctrl+Shift+Space",
            "shift+alt+KeyQ",
            "CTRL+KeyC",
            "CmdOrCtrl+Space",
            "KeyX",
            "Ctrl+5",
            "shift+f12",
        ];

        for case in valid_cases {
            let res = HotKey::from_str(case);
            assert!(
                res.is_ok(),
                "Hotkey string '{}' should have parsed successfully, but failed with: {:?}",
                case,
                res.err()
            );
        }
    }

    #[test]
    fn test_parse_invalid_hotkeys() {
        let _ = env_logger::builder().is_test(true).try_init();

        // Verify that invalid formats are rejected
        let invalid_cases = vec![
            "shift+KeyQ+alt", // Wrong modifier order
            "Shift+Ctrl",     // No main key
            "invalidkey",     // Unknown key name
            "",               // Empty string
        ];

        for case in invalid_cases {
            let res = HotKey::from_str(case);
            assert!(
                res.is_err(),
                "Hotkey string '{}' should have failed to parse, but succeeded: {:?}",
                case,
                res.ok()
            );
        }
    }
}
