//! Audio capture pipeline — cpal mic capture during PTT hold.
//! Captures at device-native sample rate, resamples to 16kHz for Whisper.
//! See architecture.md § Audio Pipeline Architecture.

use crate::error::AppError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Whisper expects 16kHz mono f32 audio.
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// Commands sent from the PTT bridge to the audio capture thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureCommand {
    Start,
    Stop,
}

pub type CaptureCommandSender = mpsc::Sender<CaptureCommand>;
pub type CaptureCommandReceiver = mpsc::Receiver<CaptureCommand>;
pub type AudioBufferSender = mpsc::Sender<Vec<f32>>;
pub type AudioBufferReceiver = mpsc::Receiver<Vec<f32>>;

/// Handle returned from `start_audio_pipeline()`.
/// Holds the command sender (to control capture) and the audio buffer receiver
/// (to receive completed audio buffers for STT).
pub struct AudioPipeline {
    pub command_tx: CaptureCommandSender,
    pub audio_rx: AudioBufferReceiver,
}

/// Resample audio from `source_rate` to 16kHz using linear interpolation.
/// Returns samples unchanged if source is already 16kHz.
pub fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Vec<f32> {
    if source_rate == WHISPER_SAMPLE_RATE {
        return samples.to_vec();
    }
    if samples.is_empty() || source_rate == 0 {
        return Vec::new();
    }

    let ratio = source_rate as f64 / WHISPER_SAMPLE_RATE as f64;
    let output_len = ((samples.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        if idx + 1 < samples.len() {
            output.push(samples[idx] * (1.0 - frac) + samples[idx + 1] * frac);
        } else if idx < samples.len() {
            output.push(samples[idx]);
        }
    }

    output
}

/// Convert i16 audio samples to f32 in [-1.0, 1.0] range.
pub fn convert_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
}

/// Start the audio capture pipeline.
///
/// Spawns a dedicated `"audio-capture"` thread that:
/// 1. Waits for `CaptureCommand::Start`
/// 2. Opens the default input device and captures audio to `Vec<f32>`
/// 3. On `CaptureCommand::Stop`, resamples to 16kHz and sends the buffer out
/// 4. Loops back to wait for the next Start
///
/// Returns an `AudioPipeline` with the command sender and audio buffer receiver.
pub fn start_audio_pipeline() -> Result<AudioPipeline, AppError> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<CaptureCommand>();
    let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();

    std::thread::Builder::new()
        .name("audio-capture".to_string())
        .spawn(move || {
            audio_capture_loop(cmd_rx, audio_tx);
        })
        .map_err(|e| AppError::Audio(format!("Failed to spawn audio thread: {}", e)))?;

    Ok(AudioPipeline {
        command_tx: cmd_tx,
        audio_rx,
    })
}

/// Main loop for the audio capture thread.
/// Blocks on `cmd_rx.recv()` when idle — zero CPU usage.
fn audio_capture_loop(cmd_rx: CaptureCommandReceiver, audio_tx: AudioBufferSender) {
    loop {
        // Block until we receive a command — zero CPU at idle
        let cmd = match cmd_rx.recv() {
            Ok(cmd) => cmd,
            Err(_) => {
                tracing::info!("Audio command channel closed — shutting down capture thread");
                return;
            }
        };

        match cmd {
            CaptureCommand::Start => {
                tracing::info!("PTT activated — starting audio capture");
                if let Err(e) = capture_until_stop(&cmd_rx, &audio_tx) {
                    tracing::error!("Audio capture error: {}", e);
                }
            }
            CaptureCommand::Stop => {
                // Stop without a preceding Start — ignore
                tracing::debug!("Received Stop without active capture — ignoring");
            }
        }
    }
}

/// Capture audio from the default input device until a `Stop` command is received.
/// On stop, resamples to 16kHz and sends the buffer via `audio_tx`.
fn capture_until_stop(
    cmd_rx: &CaptureCommandReceiver,
    audio_tx: &AudioBufferSender,
) -> Result<(), AppError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Audio("No input device found".into()))?;

    let supported_config = device
        .default_input_config()
        .map_err(|e| AppError::Audio(format!("Failed to get input config: {}", e)))?;

    let sample_rate = supported_config.sample_rate().0;
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    tracing::debug!(
        sample_rate = sample_rate,
        format = ?sample_format,
        "Audio device config"
    );

    // Shared buffer for the cpal callback to write into
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(
        sample_rate as usize * 30, // up to 30s at native rate
    )));
    let buffer_clone = Arc::clone(&buffer);

    // Build the input stream based on sample format
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        buf.extend_from_slice(data);
                    }
                },
                move |err| {
                    tracing::error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| AppError::Audio(format!("Failed to build f32 stream: {}", e)))?,
        cpal::SampleFormat::I16 => {
            device
                .build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buf) = buffer_clone.lock() {
                            buf.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                        }
                    },
                    move |err| {
                        tracing::error!("Audio stream error: {}", err);
                    },
                    None,
                )
                .map_err(|e| AppError::Audio(format!("Failed to build i16 stream: {}", e)))?
        }
        other => {
            return Err(AppError::Audio(format!(
                "Unsupported sample format: {:?}",
                other
            )));
        }
    };

    stream
        .play()
        .map_err(|e| AppError::Audio(format!("Failed to start stream: {}", e)))?;

    tracing::info!("Audio capture active");

    // Wait for Stop command
    loop {
        match cmd_rx.recv() {
            Ok(CaptureCommand::Stop) => {
                tracing::info!("PTT released — stopping audio capture");
                break;
            }
            Ok(CaptureCommand::Start) => {
                // Already capturing — ignore duplicate Start
                tracing::debug!("Received Start while already capturing — ignoring");
            }
            Err(_) => {
                tracing::info!("Command channel closed during capture");
                return Ok(());
            }
        }
    }

    // Drop stream to stop capture
    drop(stream);

    // Take the buffer, resample, and send
    let raw_buffer = {
        let mut buf = buffer.lock().map_err(|e| {
            AppError::Audio(format!("Failed to lock audio buffer: {}", e))
        })?;
        std::mem::take(&mut *buf)
    };

    if raw_buffer.is_empty() {
        tracing::warn!("Audio buffer is empty after capture — skipping send");
        return Ok(());
    }

    let resampled = resample_to_16khz(&raw_buffer, sample_rate);
    tracing::debug!(
        raw_samples = raw_buffer.len(),
        resampled_samples = resampled.len(),
        source_rate = sample_rate,
        "Audio buffer resampled to 16kHz"
    );

    // Send buffer — if receiver is dropped, that's okay (STT not wired yet)
    if audio_tx.send(resampled).is_err() {
        tracing::debug!("Audio buffer receiver dropped — discarding buffer");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_command_variants() {
        let start = CaptureCommand::Start;
        let stop = CaptureCommand::Stop;
        assert_eq!(start, CaptureCommand::Start);
        assert_eq!(stop, CaptureCommand::Stop);
        assert_ne!(start, stop);
    }

    #[test]
    fn test_resample_48khz_to_16khz() {
        // 48000 samples at 48kHz = 1 second of audio
        let source_rate = 48_000u32;
        let input: Vec<f32> = (0..source_rate as usize)
            .map(|i| (i as f32 / source_rate as f32).sin())
            .collect();

        let output = resample_to_16khz(&input, source_rate);

        // Output should be ~16000 samples (1 second at 16kHz)
        let expected_len = (input.len() as f64 * 16_000.0 / 48_000.0).ceil() as usize;
        assert_eq!(output.len(), expected_len);
    }

    #[test]
    fn test_resample_identity_at_16khz() {
        let input: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let output = resample_to_16khz(&input, WHISPER_SAMPLE_RATE);
        assert_eq!(output, input, "16kHz input should pass through unchanged");
    }

    #[test]
    fn test_resample_empty_input() {
        let output = resample_to_16khz(&[], 48_000);
        assert!(output.is_empty());
    }

    #[test]
    fn test_resample_44100_to_16khz() {
        // Common macOS sample rate
        let source_rate = 44_100u32;
        let input: Vec<f32> = vec![1.0; source_rate as usize]; // 1 second
        let output = resample_to_16khz(&input, source_rate);
        let expected_len = (source_rate as f64 / (source_rate as f64 / 16_000.0)).ceil() as usize;
        assert_eq!(output.len(), expected_len);
    }

    #[test]
    fn test_convert_i16_to_f32() {
        let input: Vec<i16> = vec![0, i16::MAX, i16::MIN, 16384];
        let output = convert_i16_to_f32(&input);
        assert_eq!(output.len(), 4);
        assert!((output[0] - 0.0).abs() < f32::EPSILON, "zero should map to 0.0");
        assert!((output[1] - 1.0).abs() < f32::EPSILON, "MAX should map to 1.0");
        assert!(output[2] < -0.99, "MIN should map to ~-1.0");
        assert!((output[3] - 0.5).abs() < 0.01, "half-MAX should map to ~0.5");
    }

    #[test]
    fn test_convert_i16_to_f32_empty() {
        let output = convert_i16_to_f32(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_audio_pipeline_channel_creation() {
        // Test that channels work for CaptureCommand round-trip
        let (tx, rx) = mpsc::channel::<CaptureCommand>();
        tx.send(CaptureCommand::Start).unwrap();
        tx.send(CaptureCommand::Stop).unwrap();
        assert_eq!(rx.recv().unwrap(), CaptureCommand::Start);
        assert_eq!(rx.recv().unwrap(), CaptureCommand::Stop);
    }

    #[test]
    fn test_audio_buffer_channel_creation() {
        // Test that Vec<f32> can be sent through the audio buffer channel
        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let buffer = vec![0.1, 0.2, 0.3];
        tx.send(buffer.clone()).unwrap();
        let received = rx.recv().unwrap();
        assert_eq!(received, buffer);
    }

    #[test]
    fn test_resample_preserves_value_range() {
        // All input values in [-1, 1] → output values should also be in [-1, 1]
        let input: Vec<f32> = (0..4800)
            .map(|i| (i as f32 * std::f32::consts::TAU / 4800.0).sin())
            .collect();
        let output = resample_to_16khz(&input, 48_000);
        for &sample in &output {
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Resampled value {} out of [-1, 1] range",
                sample
            );
        }
    }
}
