use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub provider: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_chat_model: String,
    pub groq_api_key: String,
    pub groq_model: String,
    pub groq_chat_model: String,
    pub hotkey: String,
    pub insert_mode: String,
    pub transcription_mode: String,
    pub audio_duration_limit: u32,
    pub system_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        log::debug!("Initializing Config with default values.");
        Config {
            provider: "openrouter".to_string(),
            openrouter_api_key: "".to_string(),
            openrouter_model: "google/gemini-3.1-flash-lite".to_string(),
            openai_api_key: "".to_string(),
            openai_model: "whisper-1".to_string(),
            openai_chat_model: "gpt-4o-mini".to_string(),
            groq_api_key: "".to_string(),
            groq_model: "whisper-large-v3".to_string(),
            groq_chat_model: "llama3-8b-8192".to_string(),
            hotkey: "ctrl+shift+space".to_string(),
            insert_mode: "typewriter".to_string(),
            transcription_mode: "clean".to_string(),
            audio_duration_limit: 30,
            system_prompt: "You are a pure audio transcription tool. Your ONLY task is to transcribe exactly what is spoken in the audio. Do NOT generate new text, do NOT hallucinate, do NOT complete sentences, and do NOT add any extra information. If you hear nothing or only noise, return an empty string. The audio may contain speech in Russian, English, or Uzbek. Return ONLY the transcribed text in its original language, exactly as spoken. No explanations, no translations, no prefixes.".to_string(),
        }
    }
}

impl Config {
    /// Resolves the platform-agnostic configuration directory path `~/.transcriber/`
    pub fn get_config_dir() -> PathBuf {
        let mut path = home::home_dir().unwrap_or_else(|| {
            log::warn!("Home directory not resolved, using current working directory.");
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        path.push(".transcriber");
        path
    }

    /// Resolves the platform-agnostic configuration file path `~/.transcriber/config.json`
    pub fn get_config_file_path() -> PathBuf {
        let mut path = Self::get_config_dir();
        path.push("config.json");
        path
    }

    /// Loads the configuration from disk, creating the directory and default file if they do not exist
    pub fn load() -> Self {
        let config_dir = Self::get_config_dir();
        let config_path = Self::get_config_file_path();

        log::debug!("Loading configuration from path: {:?}", config_path);

        // Ensure directory exists
        if !config_dir.exists() {
            log::info!("Configuration directory does not exist. Creating: {:?}", config_dir);
            if let Err(e) = fs::create_dir_all(&config_dir) {
                log::error!("Failed to create configuration directory: {:?}", e);
            }
        }

        // If file does not exist, write the default configuration
        if !config_path.exists() {
            log::info!("Configuration file not found. Creating default: {:?}", config_path);
            let default_config = Self::default();
            if let Err(e) = default_config.save() {
                log::error!("Failed to save default configuration: {:?}", e);
            }
            return default_config;
        }

        // Read and parse file
        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<Config>(&content) {
                Ok(config) => {
                    log::info!("Configuration loaded successfully.");
                    config
                }
                Err(e) => {
                    log::warn!("Failed to parse configuration file: {:?}. Falling back to defaults.", e);
                    let fallback_config = Self::default();
                    // We preserve the corrupted file by renaming it before rewriting to avoid losing key data
                    let mut backup_path = config_path.clone();
                    backup_path.set_extension("json.corrupted");
                    log::info!("Backing up corrupted config file to: {:?}", backup_path);
                    let _ = fs::rename(&config_path, backup_path);
                    let _ = fallback_config.save();
                    fallback_config
                }
            },
            Err(e) => {
                log::error!("Failed to read configuration file: {:?}. Falling back to defaults.", e);
                Self::default()
            }
        }
    }

    /// Saves the current configuration to `~/.transcriber/config.json`
    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_path = Self::get_config_file_path();
        log::debug!("Saving configuration to path: {:?}", config_path);

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&config_path, content)?;
        log::info!("Configuration saved successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config_fields() {
        // We need a dummy logger to ensure log statements compile during tests
        let _ = env_logger::builder().is_test(true).try_init();

        let config = Config::default();
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.openrouter_model, "google/gemini-3.1-flash-lite");
        assert_eq!(config.audio_duration_limit, 30);
        assert!(config.system_prompt.contains("pure audio transcription"));
    }

    #[test]
    fn test_load_and_save_with_env_override() {
        let _ = env_logger::builder().is_test(true).try_init();

        let temp_home = std::env::temp_dir().join("transcriber_test_home_io");
        let orig_userprofile = std::env::var("USERPROFILE").ok();
        let orig_home = std::env::var("HOME").ok();

        std::env::set_var("USERPROFILE", &temp_home);
        std::env::set_var("HOME", &temp_home);

        // Clear existing test directory if any
        let config_dir = Config::get_config_dir();
        let _ = fs::remove_dir_all(&config_dir);

        // 1. Loading when nothing exists should return default and write it to disk
        let config = Config::load();
        assert_eq!(config.provider, "openrouter");
        assert!(Config::get_config_file_path().exists());

        // 2. Modifying and saving
        let mut config = config;
        config.provider = "groq".to_string();
        config.save().unwrap();

        // 3. Loading again should return the modified config
        let reloaded = Config::load();
        assert_eq!(reloaded.provider, "groq");

        // 4. Loading corrupted file should backup to .corrupted and load defaults
        let config_file = Config::get_config_file_path();
        fs::write(&config_file, "invalid json data").unwrap();

        let corrupted_loaded = Config::load();
        assert_eq!(corrupted_loaded.provider, "openrouter"); // fallbacks
        
        let mut corrupted_backup = config_file.clone();
        corrupted_backup.set_extension("json.corrupted");
        assert!(corrupted_backup.exists());

        // Restore original env vars
        if let Some(val) = orig_userprofile {
            std::env::set_var("USERPROFILE", val);
        } else {
            std::env::remove_var("USERPROFILE");
        }
        if let Some(val) = orig_home {
            std::env::set_var("HOME", val);
        } else {
            std::env::remove_var("HOME");
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_home);
    }
}
