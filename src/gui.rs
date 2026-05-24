use crate::config::Config;
use crate::orchestrator::AppEvent;
use egui::{Color32, Context, Vec2, ViewportBuilder, ViewportCommand, Visuals};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static SETTINGS_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    ApiSettings,
    BehaviorSettings,
}

pub struct SettingsApp {
    config: Arc<Mutex<Config>>,
    event_sender: std::sync::mpsc::Sender<AppEvent>,
    local_config: Config,
    active_tab: Tab,
    openrouter_key_visible: bool,
    openai_key_visible: bool,
    groq_key_visible: bool,
}

impl SettingsApp {
    pub fn new(
        config: Arc<Mutex<Config>>,
        event_sender: std::sync::mpsc::Sender<AppEvent>,
    ) -> Self {
        let local_config = {
            let guard = config.lock().expect("Failed to lock config for GUI startup");
            guard.clone()
        };

        Self {
            config,
            event_sender,
            local_config,
            active_tab: Tab::ApiSettings,
            openrouter_key_visible: false,
            openai_key_visible: false,
            groq_key_visible: false,
        }
    }
}

pub fn apply_custom_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();

    // Custom Visual Slate Palette
    visuals.window_fill = Color32::from_rgb(0x0D, 0x0F, 0x12); // Deep dark slate
    visuals.faint_bg_color = Color32::from_rgb(0x1A, 0x1C, 0x23); // Inner GroupBox
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x15, 0x17, 0x1C); // Input Fields
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x23, 0x26, 0x31);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0xC0, 0x84, 0xFC); // Accent Highlights

    style.visuals = visuals;
    ctx.set_style(style);
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Enforce settings window is marked as open
        SETTINGS_WINDOW_OPEN.store(true, Ordering::SeqCst);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.heading("🎙 Transcriber Settings");
                ui.add_space(4.0);
            });

            // Dual Tab Selector
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let api_btn = ui.selectable_label(
                    self.active_tab == Tab::ApiSettings,
                    " 🔑 API Credentials ",
                );
                if api_btn.clicked() {
                    self.active_tab = Tab::ApiSettings;
                }

                let behavior_btn = ui.selectable_label(
                    self.active_tab == Tab::BehaviorSettings,
                    " ⚙ App Behavior ",
                );
                if behavior_btn.clicked() {
                    self.active_tab = Tab::BehaviorSettings;
                }
            });
            ui.separator();
            ui.add_space(8.0);

            // Tab Content Frame
            egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                match self.active_tab {
                    Tab::ApiSettings => {
                        ui.group(|ui| {
                            ui.label("🎯 Active API Provider:");
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.local_config.provider,
                                    "openrouter".to_string(),
                                    "OpenRouter",
                                );
                                ui.radio_value(
                                    &mut self.local_config.provider,
                                    "openai".to_string(),
                                    "OpenAI (Whisper)",
                                );
                                ui.radio_value(
                                    &mut self.local_config.provider,
                                    "groq".to_string(),
                                    "Groq (Whisper)",
                                );
                            });
                        });
                        ui.add_space(10.0);

                        // OpenRouter Settings Group
                        ui.group(|ui| {
                            ui.heading("🚀 OpenRouter Configuration");
                            ui.add_space(4.0);

                            ui.label("Model:");
                            ui.text_edit_singleline(&mut self.local_config.openrouter_model);

                            ui.label("API Key:");
                            ui.horizontal(|ui| {
                                if self.openrouter_key_visible {
                                    ui.text_edit_singleline(&mut self.local_config.openrouter_api_key);
                                } else {
                                    ui.add(
                                        egui::TextEdit::singleline(
                                            &mut self.local_config.openrouter_api_key,
                                        )
                                        .password(true),
                                    );
                                }
                                if ui
                                    .button(if self.openrouter_key_visible { "👁" } else { "🙈" })
                                    .on_hover_text("Toggle API key visibility")
                                    .clicked()
                                {
                                    self.openrouter_key_visible = !self.openrouter_key_visible;
                                }
                            });
                        });
                        ui.add_space(10.0);

                        // OpenAI Settings Group
                        ui.group(|ui| {
                            ui.heading("🟢 OpenAI Configuration");
                            ui.add_space(4.0);

                            ui.label("Whisper Audio Model:");
                            ui.text_edit_singleline(&mut self.local_config.openai_model);

                            ui.label("Chat Processing Model:");
                            ui.text_edit_singleline(&mut self.local_config.openai_chat_model);

                            ui.label("API Key:");
                            ui.horizontal(|ui| {
                                if self.openai_key_visible {
                                    ui.text_edit_singleline(&mut self.local_config.openai_api_key);
                                } else {
                                    ui.add(
                                        egui::TextEdit::singleline(
                                            &mut self.local_config.openai_api_key,
                                        )
                                        .password(true),
                                    );
                                }
                                if ui
                                    .button(if self.openai_key_visible { "👁" } else { "🙈" })
                                    .on_hover_text("Toggle API key visibility")
                                    .clicked()
                                {
                                    self.openai_key_visible = !self.openai_key_visible;
                                }
                            });
                        });
                        ui.add_space(10.0);

                        // Groq Settings Group
                        ui.group(|ui| {
                            ui.heading("🟠 Groq Configuration");
                            ui.add_space(4.0);

                            ui.label("Whisper Audio Model:");
                            ui.text_edit_singleline(&mut self.local_config.groq_model);

                            ui.label("Chat Processing Model:");
                            ui.text_edit_singleline(&mut self.local_config.groq_chat_model);

                            ui.label("API Key:");
                            ui.horizontal(|ui| {
                                if self.groq_key_visible {
                                    ui.text_edit_singleline(&mut self.local_config.groq_api_key);
                                } else {
                                    ui.add(
                                        egui::TextEdit::singleline(
                                            &mut self.local_config.groq_api_key,
                                        )
                                        .password(true),
                                    );
                                }
                                if ui
                                    .button(if self.groq_key_visible { "👁" } else { "🙈" })
                                    .on_hover_text("Toggle API key visibility")
                                    .clicked()
                                {
                                    self.groq_key_visible = !self.groq_key_visible;
                                }
                            });
                        });
                    }
                    Tab::BehaviorSettings => {
                        // General App Settings Group
                        ui.group(|ui| {
                            ui.heading("🎮 Global Hotkey & Output");
                            ui.add_space(4.0);

                            ui.label("Trigger Hotkey:");
                            let _hotkey_box = ui.text_edit_singleline(&mut self.local_config.hotkey);
                            let parsed = <global_hotkey::hotkey::HotKey as std::str::FromStr>::from_str(&self.local_config.hotkey);
                            if let Err(e) = parsed {
                                ui.colored_label(
                                    Color32::from_rgb(239, 68, 68),
                                    format!("⚠️ Invalid hotkey: {:?}", e),
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(34, 197, 94),
                                    "✨ Hotkey is valid",
                                );
                            }

                            ui.add_space(4.0);
                            ui.label("Output Injection Method:");
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.local_config.insert_mode,
                                    "typewriter".to_string(),
                                    "Typewriter Emulation",
                                );
                                ui.radio_value(
                                    &mut self.local_config.insert_mode,
                                    "clipboard".to_string(),
                                    "Clipboard Inject Only",
                                );
                            });

                            ui.add_space(4.0);
                            ui.label("Transcription Processing Mode:");
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.local_config.transcription_mode,
                                    "clean".to_string(),
                                    "Clean Transcription",
                                );
                                ui.radio_value(
                                    &mut self.local_config.transcription_mode,
                                    "translate".to_string(),
                                    "Auto-Translate to English",
                                );
                            });

                            ui.add_space(6.0);
                            ui.label("Max Audio Capture Duration (seconds):");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.local_config.audio_duration_limit,
                                    5..=120,
                                )
                                .text("sec"),
                            );
                        });
                        ui.add_space(10.0);

                        // Prompt Customization Group
                        ui.group(|ui| {
                            ui.heading("✍️ System Instruction Prompt");
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::multiline(&mut self.local_config.system_prompt)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(6)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();

            // Footer Actions
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_hotkey_valid = <global_hotkey::hotkey::HotKey as std::str::FromStr>::from_str(&self.local_config.hotkey).is_ok();

                // Save button
                ui.add_enabled_ui(is_hotkey_valid, |ui| {
                    let save_btn = ui.button("💾 Save Settings");
                    if save_btn.clicked() {
                        log::info!("Saving updated configuration settings dynamically.");
                        if let Err(e) = self.local_config.save() {
                            log::error!("Failed to save modified configuration to disk: {:?}", e);
                        } else {
                            // Update shared configuration
                            {
                                let mut guard = self.config.lock().expect("Failed to lock global config");
                                *guard = self.local_config.clone();
                            }
                            // Notify event orchestrator
                            let _ = self
                                .event_sender
                                .send(AppEvent::ConfigUpdated(Box::new(self.local_config.clone())));
                        }
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });

                // Cancel button
                let cancel_btn = ui.button("❌ Cancel");
                if cancel_btn.clicked() {
                    log::info!("Settings change discarded by user.");
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            });
        });
    }
}

pub fn open_settings_window(
    config: Arc<Mutex<Config>>,
    event_sender: std::sync::mpsc::Sender<AppEvent>,
) {
    // Thread safe check to ensure only one settings window is active at any time.
    if SETTINGS_WINDOW_OPEN
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        log::warn!("Settings window is already active. Ignoring spawn request.");
        return;
    }

    // Load custom window icon
    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon_data = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Png)
        .ok()
        .map(|img| {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }
        });

    let mut viewport = ViewportBuilder::default()
        .with_inner_size(Vec2::new(550.0, 520.0))
        .with_resizable(true)
        .with_title("Transcriber Settings");

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        run_and_return: true,
        ..Default::default()
    };

    let result = eframe::run_native(
        "Transcriber Settings",
        options,
        Box::new(move |cc| {
            apply_custom_theme(&cc.egui_ctx);
            Box::new(SettingsApp::new(config, event_sender))
        }),
    );

    if let Err(e) = result {
        log::error!("Failed to run eframe Native settings window: {:?}", e);
    }

    // Release global lock on window closed
    SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
    log::info!("Settings window closed. Resource allocations reclaimed.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_settings_app_initialization() {
        let (sender, _receiver) = mpsc::channel();
        let config = Arc::new(Mutex::new(Config::default()));
        let app = SettingsApp::new(config, sender);

        assert_eq!(app.active_tab, Tab::ApiSettings);
        assert_eq!(app.local_config.provider, "openrouter");
        assert_eq!(app.local_config.audio_duration_limit, 30);
    }
}
