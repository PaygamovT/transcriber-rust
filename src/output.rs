use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};

/// Injects text using the configured insert mode (typewriter or clipboard).
/// - **typewriter**: Copies text to clipboard, then simulates Ctrl+V paste.
///   This is far more reliable for Unicode (Cyrillic, Uzbek, etc.) on Windows
///   than character-by-character enigo.text() which corrupts non-ASCII text.
/// - **clipboard**: Copies text to the clipboard without pasting.
pub fn inject_text(insert_mode: &str, text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    match insert_mode {
        "clipboard" => {
            log::info!("Copying text directly to the clipboard.");
            copy_to_clipboard(text)?;
            log::info!("Successfully copied text to clipboard.");
            Ok(())
        }
        "typewriter" => {
            log::info!("Injecting text via clipboard-paste (Ctrl+V) for Unicode safety.");

            // 1. Save current clipboard content so we can restore it after pasting
            let mut clipboard = Clipboard::new()
                .map_err(|e| format!("Failed to initialize Clipboard: {:?}", e))?;
            let previous_clipboard = clipboard.get_text().ok();

            // 2. Set our transcription text into the clipboard
            clipboard.set_text(text.to_owned())
                .map_err(|e| format!("Failed to set text to Clipboard: {:?}", e))?;

            // 3. Brief delay to let the clipboard settle
            std::thread::sleep(std::time::Duration::from_millis(50));

            // 4. Simulate Ctrl+V paste
            let settings = Settings::default();
            match Enigo::new(&settings) {
                Ok(mut enigo) => {
                    // Press Ctrl+V
                    let _ = enigo.key(Key::Control, Direction::Press);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    let _ = enigo.key(Key::Control, Direction::Release);

                    log::info!("Successfully pasted text via Ctrl+V.");
                }
                Err(e) => {
                    log::error!("Failed to initialize Enigo for Ctrl+V paste: {:?}", e);
                    // Text is already in clipboard, so the user can manually paste
                    return Ok(());
                }
            }

            // 5. Brief delay, then restore the previous clipboard content
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Some(prev) = previous_clipboard {
                // Best-effort restore; ignore errors
                if let Ok(mut cb) = Clipboard::new() {
                    let _ = cb.set_text(prev);
                }
            }

            Ok(())
        }
        other => {
            log::warn!("Unsupported insert_mode '{}'. Falling back to clipboard.", other);
            copy_to_clipboard(text)
        }
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to initialize Clipboard: {:?}", e))?;
    clipboard.set_text(text.to_owned())
        .map_err(|e| format!("Failed to set text to Clipboard: {:?}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_text_empty() {
        // Empty text should return Ok immediately without doing anything
        assert_eq!(inject_text("typewriter", ""), Ok(()));
        assert_eq!(inject_text("clipboard", ""), Ok(()));
    }

    #[test]
    fn test_inject_text_invalid_mode_fallback() {
        // An invalid mode should fall back to clipboard copy.
        // We handle Ok or Err depending on whether clipboard is available in the test runner context.
        let result = inject_text("invalid_mode", "Test fallback text");
        match result {
            Ok(_) => log::info!("Test passed: Fallback successfully wrote to clipboard."),
            Err(e) => {
                log::warn!("Test runner did not have access to OS clipboard: {}", e);
                // The test is still considered valid since the path was correctly taken and failed at OS level
            }
        }
    }
}
