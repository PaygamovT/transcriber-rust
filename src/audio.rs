use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use hound::{WavSpec, WavWriter};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Context returned upon successfully starting audio recording.
pub struct RecordingContext {
    pub stream: cpal::Stream,
    pub buffer: Arc<Mutex<Vec<f32>>>,
    pub sample_rate: u32,
}

/// Starts capturing microphone audio in real-time on a background thread.
/// Downmixes multi-channel input to mono and returns the RecordingContext containing
/// the CPAL Stream, the shared sample buffer, and the stream's native sample rate.
pub fn start_recording() -> Result<RecordingContext, String> {
    log::info!("Initializing default host and querying default input microphone.");

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No default audio input device available. Please connect a microphone.".to_string())?;

    let device_name = device
        .name()
        .unwrap_or_else(|_| "Unknown Device".to_string());
    log::info!("Selected audio input device: '{}'", device_name);

    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to retrieve default input configuration: {:?}", e))?;

    let stream_config: cpal::StreamConfig = config.clone().into();
    let channels = stream_config.channels as usize;
    let native_sample_rate = stream_config.sample_rate.0;

    log::info!(
        "Configuring CPAL stream: channels={}, sample_rate={}, format={:?}",
        channels,
        native_sample_rate,
        config.sample_format()
    );

    let audio_buffer = Arc::new(Mutex::new(Vec::new()));
    let audio_buffer_clone = audio_buffer.clone();

    let error_callback = |err| {
        log::error!("Asynchronous CPAL input stream error: {:?}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let buffer = audio_buffer_clone;
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| {
                    if let Ok(mut buf) = buffer.try_lock() {
                        for frame in data.chunks(channels) {
                            let sum: f32 = frame.iter().sum();
                            buf.push(sum / channels as f32);
                        }
                    }
                },
                error_callback,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let buffer = audio_buffer_clone;
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| {
                    if let Ok(mut buf) = buffer.try_lock() {
                        for frame in data.chunks(channels) {
                            let mut sum = 0.0f32;
                            for &s in frame {
                                sum += s.to_sample::<f32>();
                            }
                            buf.push(sum / channels as f32);
                        }
                    }
                },
                error_callback,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let buffer = audio_buffer_clone;
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &_| {
                    if let Ok(mut buf) = buffer.try_lock() {
                        for frame in data.chunks(channels) {
                            let mut sum = 0.0f32;
                            for &s in frame {
                                sum += s.to_sample::<f32>();
                            }
                            buf.push(sum / channels as f32);
                        }
                    }
                },
                error_callback,
                None,
            )
        }
        fmt => return Err(format!("Unsupported hardware sample format: {:?}", fmt)),
    }
    .map_err(|e| format!("Failed to build CPAL input stream: {:?}", e))?;

    log::debug!("Starting audio capture stream...");
    stream
        .play()
        .map_err(|e| format!("Failed to start CPAL audio stream: {:?}", e))?;

    log::info!("🎤 Microphone stream successfully activated and capturing.");
    Ok(RecordingContext {
        stream,
        buffer: audio_buffer,
        sample_rate: native_sample_rate,
    })
}

/// Software linear resampling interpolator.
/// Downmixes/resamples a buffer of `f32` samples from `from_rate` to `to_rate`.
pub fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }

    log::debug!("Resampling audio stream from {} Hz to {} Hz.", from_rate, to_rate);

    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio).floor() as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let pos = i as f64 * ratio;
        let index = pos.floor() as usize;
        let frac = pos - index as f64;

        if index + 1 < samples.len() {
            let s0 = samples[index];
            let s1 = samples[index + 1];
            let interpolated = s0 + (s1 - s0) * frac as f32;
            resampled.push(interpolated);
        } else if index < samples.len() {
            resampled.push(samples[index]);
        }
    }

    resampled
}

/// Normalizes the peak absolute amplitude of a `f32` buffer to `0.9` and scales
/// values into the range of standard signed 16-bit integer PCM (`i16`).
pub fn normalize_and_convert(samples: &[f32]) -> Vec<i16> {
    if samples.is_empty() {
        return Vec::new();
    }

    // 1. Find absolute peak amplitude
    let mut peak = 0.0f32;
    for &s in samples {
        let abs_s = s.abs();
        if abs_s > peak {
            peak = abs_s;
        }
    }

    // 2. Compute scaling factor to hit 0.9 peak
    let scale = if peak > 0.0 { 0.9 / peak } else { 1.0 };

    log::debug!(
        "Peak absolute amplitude: {:.4}, Scaling factor applied: {:.4}",
        peak,
        scale
    );

    // 3. Apply scale, clamp, and project to i16 bounds symmetrically
    let mut converted = Vec::with_capacity(samples.len());
    for &s in samples {
        let scaled = s * scale;
        let clamped = scaled.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * 32767.0) as i16;
        converted.push(sample_i16);
    }

    converted
}

/// Encodes a slice of 16-bit PCM integer samples to a WAV byte stream entirely in RAM.
/// Configures the resulting WAV stream to exactly 16000 Hz, 1 Channel (mono), 16-bit.
pub fn encode_wav_in_memory(samples: &[i16]) -> Result<Vec<u8>, String> {
    log::debug!("Compiling {} PCM i16 samples into in-memory WAV byte buffer.", samples.len());

    let mut cursor = Cursor::new(Vec::new());
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    {
        let mut writer = WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create Hound WavWriter: {:?}", e))?;

        for &sample in samples {
            writer
                .write_sample(sample)
                .map_err(|e| format!("Failed to write audio sample to WAV: {:?}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize Hound WavWriter: {:?}", e))?;
    }

    let wav_bytes = cursor.into_inner();
    log::info!("WAV stream successfully compiled (Buffer Size: {} bytes).", wav_bytes.len());
    Ok(wav_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_resampling_ratio() {
        let _ = env_logger::builder().is_test(true).try_init();

        // 1. Generate a 1-second 440Hz sine wave at 48000 Hz
        let sample_rate_48k = 48000;
        let mut samples = Vec::new();
        for i in 0..sample_rate_48k {
            let t = i as f32 / sample_rate_48k as f32;
            samples.push((t * 440.0 * 2.0 * PI).sin());
        }

        // 2. Resample to 16000 Hz
        let resampled = resample(&samples, sample_rate_48k, 16000);

        // 3. Confirm target length is exactly 1/3 of the original
        assert_eq!(resampled.len(), 16000);

        // 4. Verify that bounds remain bounded
        for &s in &resampled {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_normalization_scaling() {
        let _ = env_logger::builder().is_test(true).try_init();

        // 1. Create a buffer with max amplitude of 0.5
        let samples = vec![-0.5f32, 0.1, 0.3, 0.5, -0.2];

        // 2. Normalize and convert
        let converted = normalize_and_convert(&samples);

        // 3. The peak absolute value (0.5) should be scaled to exactly 0.9 of i16::MAX (32767)
        // 0.9 * 32767 = 29490
        let max_val = converted.iter().map(|&x| x.abs()).max().unwrap();
        assert!(
            (max_val - 29490).abs() <= 5,
            "Peak value should be approx 29490, got {}",
            max_val
        );
    }

    #[test]
    fn test_wav_encoding_roundtrip() {
        let _ = env_logger::builder().is_test(true).try_init();

        // 1. Create dummy i16 samples
        let samples = vec![0i16, 1000, -1000, 2000, -2000, 5000, -5000];

        // 2. Encode to in-memory WAV bytes
        let wav_bytes = encode_wav_in_memory(&samples).unwrap();

        // 3. Ensure WAV header prefix exists (RIFF + WAVE)
        assert!(wav_bytes.len() > 44);
        assert_eq!(&wav_bytes[0..4], b"RIFF");
        assert_eq!(&wav_bytes[8..12], b"WAVE");

        // 4. Load using Hound's WavReader to verify roundtrip parsing correctness
        let cursor = Cursor::new(wav_bytes);
        let mut reader = hound::WavReader::new(cursor).unwrap();
        let spec = reader.spec();

        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 16000);
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        let parsed_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(parsed_samples, samples);
    }
}
