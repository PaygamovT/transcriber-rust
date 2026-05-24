use serde::{Deserialize, Serialize};
use base64::Engine;

#[derive(Serialize)]
struct OpenRouterMessageContentPart {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_audio: Option<InputAudioPart>,
}

#[derive(Serialize)]
struct InputAudioPart {
    data: String,
    format: String,
}

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: Vec<OpenRouterMessageContentPart>,
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Deserialize)]
struct ChatCompletionMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct WhisperResponse {
    text: String,
}

#[derive(Serialize)]
struct ChatCompletionMessageInput {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessageInput>,
}

/// Transcribes in-memory WAV audio bytes using the configured API provider and processing mode.
pub fn transcribe_audio(config: &crate::config::Config, wav_bytes: &[u8]) -> Result<String, String> {
    if wav_bytes.is_empty() {
        return Err("Audio buffer is empty. Nothing to transcribe.".to_string());
    }

    match config.provider.as_str() {
        "openrouter" => {
            // Select optimized prompt depending on transcription mode
            let system_prompt = match config.transcription_mode.as_str() {
                "clean" => "You are a professional audio transcription assistant. Transcribe the audio exactly as spoken, but clean it up by removing stutters, stammers, and filler words (like 'um', 'uh', 'like', 'ну', 'так сказать'). Keep the original language (Russian, English, or Uzbek). Output ONLY the refined transcription. No explanations.",
                "translate" => "You are a professional audio translator. Translate the speech in the audio directly into fluent, natural English. Output ONLY the English translation. No explanations, no prefixes.",
                _ => &config.system_prompt,
            };
            transcribe_openrouter(config, system_prompt, wav_bytes)
        }
        "openai" => {
            let is_translate = config.transcription_mode == "translate";
            let endpoint = if is_translate {
                "https://api.openai.com/v1/audio/translations"
            } else {
                "https://api.openai.com/v1/audio/transcriptions"
            };

            let raw_text = transcribe_whisper(
                config,
                endpoint,
                &config.openai_api_key,
                &config.openai_model,
                wav_bytes,
            )?;

            if config.transcription_mode == "clean" {
                clean_transcription_with_llm(
                    "https://api.openai.com/v1/chat/completions",
                    &config.openai_api_key,
                    &config.openai_chat_model,
                    &raw_text,
                )
            } else {
                Ok(raw_text)
            }
        }
        "groq" => {
            let is_translate = config.transcription_mode == "translate";
            let endpoint = if is_translate {
                "https://api.groq.com/openai/v1/audio/translations"
            } else {
                "https://api.groq.com/openai/v1/audio/transcriptions"
            };

            let raw_text = transcribe_whisper(
                config,
                endpoint,
                &config.groq_api_key,
                &config.groq_model,
                wav_bytes,
            )?;

            if config.transcription_mode == "clean" {
                clean_transcription_with_llm(
                    "https://api.groq.com/openai/v1/chat/completions",
                    &config.groq_api_key,
                    &config.groq_chat_model,
                    &raw_text,
                )
            } else {
                Ok(raw_text)
            }
        }
        other => Err(format!("Unsupported transcription provider: '{}'", other)),
    }
}

fn transcribe_openrouter(config: &crate::config::Config, system_prompt: &str, wav_bytes: &[u8]) -> Result<String, String> {
    if config.openrouter_api_key.trim().is_empty() {
        return Err("OpenRouter API Key is empty. Please configure it in the settings panel.".to_string());
    }

    log::info!("Preparing OpenRouter multimodal speech-to-text request.");

    let base64_audio = base64::prelude::BASE64_STANDARD.encode(wav_bytes);

    let request_payload = OpenRouterRequest {
        model: config.openrouter_model.clone(),
        messages: vec![OpenRouterMessage {
            role: "user".to_string(),
            content: vec![
                OpenRouterMessageContentPart {
                    part_type: "text".to_string(),
                    text: Some(system_prompt.to_string()),
                    input_audio: None,
                },
                OpenRouterMessageContentPart {
                    part_type: "input_audio".to_string(),
                    text: None,
                    input_audio: Some(InputAudioPart {
                        data: base64_audio,
                        format: "wav".to_string(),
                    }),
                },
            ],
        }],
    };

    let response = ureq::post("https://openrouter.ai/api/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", config.openrouter_api_key))
        .set("Content-Type", "application/json")
        .set("HTTP-Referer", "https://github.com/tolib/TranscriberRUST")
        .set("X-Title", "Transcriber RUST")
        .send_json(serde_json::to_value(&request_payload).map_err(|e| format!("Serialization error: {}", e))?)
        .map_err(|err| match err {
            ureq::Error::Status(code, resp) => {
                let err_msg = resp.into_string().unwrap_or_else(|_| "Unknown error".to_string());
                format!("OpenRouter API error (Status {}): {}", code, err_msg)
            }
            ureq::Error::Transport(transport) => {
                format!("OpenRouter network transport error: {:?}", transport)
            }
        })?;

    let parsed: ChatCompletionResponse = response
        .into_json()
        .map_err(|e| format!("Failed to parse OpenRouter JSON response: {}", e))?;

    if let Some(choice) = parsed.choices.first() {
        if let Some(ref content) = choice.message.content {
            let trimmed = content.trim().to_string();
            log::info!("Successfully received transcription from OpenRouter (Length: {} chars).", trimmed.len());
            return Ok(trimmed);
        }
    }

    Err("OpenRouter returned a response with no text content choice.".to_string())
}

/// Helper function to construct manual multipart/form-data payload.
/// Exposed for testing purposes.
pub fn build_multipart_payload(
    boundary: &str,
    model: &str,
    prompt: &str,
    wav_bytes: &[u8],
) -> Vec<u8> {
    let mut body = Vec::new();

    // 1. Model field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"model\"\r\n\r\n");
    body.extend_from_slice(format!("{}\r\n", model).as_bytes());

    // 2. Prompt field
    if !prompt.trim().is_empty() {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"prompt\"\r\n\r\n");
        body.extend_from_slice(format!("{}\r\n", prompt).as_bytes());
    }

    // 3. Audio file field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"speech.wav\"\r\n");
    body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
    body.extend_from_slice(wav_bytes);
    body.extend_from_slice(b"\r\n");

    // 4. Closing delimiter
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    body
}

fn transcribe_whisper(
    config: &crate::config::Config,
    endpoint: &str,
    api_key: &str,
    model: &str,
    wav_bytes: &[u8],
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("API Key is empty. Please configure it in the settings panel.".to_string());
    }

    log::info!("Preparing Whisper multipart/form-data transcription request to {}", endpoint);

    let boundary = "----TranscriberRustBoundaryMultipartField123456789";
    let body = build_multipart_payload(boundary, model, &config.system_prompt, wav_bytes);

    let response = ureq::post(endpoint)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", &format!("multipart/form-data; boundary={}", boundary))
        .send_bytes(&body)
        .map_err(|err| match err {
            ureq::Error::Status(code, resp) => {
                let err_msg = resp.into_string().unwrap_or_else(|_| "Unknown error".to_string());
                format!("Whisper API error (Status {}): {}", code, err_msg)
            }
            ureq::Error::Transport(transport) => {
                format!("Whisper network transport error: {:?}", transport)
            }
        })?;

    let parsed: WhisperResponse = response
        .into_json()
        .map_err(|e| format!("Failed to parse Whisper JSON response: {}", e))?;

    let trimmed = parsed.text.trim().to_string();
    log::info!("Successfully received transcription from Whisper API (Length: {} chars).", trimmed.len());
    Ok(trimmed)
}

fn clean_transcription_with_llm(
    endpoint: &str,
    api_key: &str,
    model: &str,
    raw_text: &str,
) -> Result<String, String> {
    if raw_text.trim().is_empty() {
        return Ok(String::new());
    }

    log::info!("Cleaning up transcription via LLM Chat Completion using model: {}", model);

    let system_instruction = "You are an expert text editor. Clean up this speech transcription by removing stutters, filler words (like 'um', 'uh', 'like', 'you know', 'ну', 'это', 'так сказать'), and repeating words. Keep the original language, punctuation, and core meaning. Output ONLY the cleaned text with no explanations or metadata.";

    let request_payload = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![
            ChatCompletionMessageInput {
                role: "system".to_string(),
                content: system_instruction.to_string(),
            },
            ChatCompletionMessageInput {
                role: "user".to_string(),
                content: raw_text.to_string(),
            },
        ],
    };

    let response = ureq::post(endpoint)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(serde_json::to_value(&request_payload).map_err(|e| format!("Serialization error: {}", e))?)
        .map_err(|err| match err {
            ureq::Error::Status(code, resp) => {
                let err_msg = resp.into_string().unwrap_or_else(|_| "Unknown error".to_string());
                format!("Chat Completion API error (Status {}): {}", code, err_msg)
            }
            ureq::Error::Transport(transport) => {
                format!("Chat Completion network transport error: {:?}", transport)
            }
        })?;

    let parsed: ChatCompletionResponse = response
        .into_json()
        .map_err(|e| format!("Failed to parse Chat Completion JSON response: {}", e))?;

    if let Some(choice) = parsed.choices.first() {
        if let Some(ref content) = choice.message.content {
            return Ok(content.trim().to_string());
        }
    }

    Err("Chat Completion API returned a response with no text content choice.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_multipart_builder() {
        let boundary = "TestBoundary123";
        let model = "whisper-1";
        let prompt = "Transcribe exactly.";
        let wav_data = b"RIFF....WAVEfmt....data...";

        let payload = build_multipart_payload(boundary, model, prompt, wav_data);
        let payload_str = String::from_utf8_lossy(&payload);

        // 1. Verify model is written correctly
        assert!(payload_str.contains("--TestBoundary123"));
        assert!(payload_str.contains("Content-Disposition: form-data; name=\"model\""));
        assert!(payload_str.contains("whisper-1"));

        // 2. Verify prompt is written correctly
        assert!(payload_str.contains("Content-Disposition: form-data; name=\"prompt\""));
        assert!(payload_str.contains("Transcribe exactly."));

        // 3. Verify file field and header
        assert!(payload_str.contains("Content-Disposition: form-data; name=\"file\"; filename=\"speech.wav\""));
        assert!(payload_str.contains("Content-Type: audio/wav"));

        // 4. Verify binary data injection exists
        assert!(payload.windows(wav_data.len()).any(|window| window == wav_data));

        // 5. Verify closing boundary exists
        assert!(payload_str.contains("--TestBoundary123--"));
    }

    #[test]
    fn test_chat_completion_payload_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![ChatCompletionMessageInput {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains(r#""model":"gpt-4o-mini""#));
        assert!(serialized.contains(r#""role":"user""#));
        assert!(serialized.contains(r#""content":"Hello""#));
    }
}
