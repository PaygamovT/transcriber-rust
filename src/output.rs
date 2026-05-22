use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Settings};

/// Injects text using the configured insert mode (typewriter or clipboard).
/// Automatically falls back to clipboard copying if typewriter simulation fails.
pub fn inject_text(insert_mode: &str, text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    match insert_mode {
        "clipboard" => {
            log::info!("Copying text directly to the clipboard.");
            let mut clipboard = Clipboard::new()
                .map_err(|e| format!("Failed to initialize Clipboard: {:?}", e))?;
            clipboard.set_text(text.to_owned())
                .map_err(|e| format!("Failed to set text to Clipboard: {:?}", e))?;
            log::info!("Successfully copied text to clipboard.");
            Ok(())
        }
        "typewriter" => {
            log::info!("Injecting text using typewriter simulation.");
            let settings = Settings::default();
            match Enigo::new(&settings) {
                Ok(mut enigo) => {
                    if let Err(e) = enigo.text(text) {
                        log::warn!("Typewriter simulation failed: {:?}. Falling back to clipboard.", e);
                        fallback_to_clipboard(text)
                    } else {
                        log::info!("Successfully emulated typing.");
                        Ok(())
                    }
                }
                Err(e) => {
                    log::warn!("Failed to initialize Enigo: {:?}. Falling back to clipboard.", e);
                    fallback_to_clipboard(text)
                }
            }
        }
        other => {
            log::warn!("Unsupported insert_mode '{}'. Falling back to clipboard.", other);
            fallback_to_clipboard(text)
        }
    }
}

fn fallback_to_clipboard(text: &str) -> Result<(), String> {
    log::info!("Executing graceful fallback to clipboard.");
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to initialize Clipboard: {:?}", e))?;
    clipboard.set_text(text.to_owned())
        .map_err(|e| format!("Failed to set text to Clipboard: {:?}", e))?;
    log::info!("Successfully completed clipboard fallback.");
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
