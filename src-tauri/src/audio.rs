use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};
use serde::Serialize;
use std::io::Cursor;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use crate::error::AppError;

const TARGET_SAMPLE_RATE: u32 = 16000;
const TARGET_CHANNELS: u16 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    pub index: usize,
    pub name: String,
}

pub fn list_input_devices() -> Result<Vec<AudioDevice>, AppError> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    for (index, device) in host
        .input_devices()
        .map_err(|e| AppError::Audio(e.to_string()))?
        .enumerate()
    {
        let name = device.name().unwrap_or_else(|_| format!("Device {}", index));

        // Filter out low-level ALSA devices that clutter the list on Linux.
        // Keep only pipewire/pulse/default entries which are the usable ones.
        let lower = name.to_lowercase();
        if lower.starts_with("hw:")
            || lower.starts_with("plughw:")
            || lower.starts_with("sysdefault:")
            || lower.starts_with("front:")
            || lower.starts_with("rear:")
            || lower.starts_with("center_lfe:")
            || lower.starts_with("side:")
            || lower.starts_with("surround")
            || lower.starts_with("iec958:")
            || lower.starts_with("spdif:")
            || lower.starts_with("hdmi:")
            || lower.starts_with("dmix:")
            || lower.starts_with("dsnoop:")
            || lower.starts_with("null")
        {
            continue;
        }

        devices.push(AudioDevice { index, name });
    }

    Ok(devices)
}

fn get_input_device(device_index: Option<usize>) -> Result<Device, AppError> {
    let host = cpal::default_host();

    match device_index {
        Some(idx) => host
            .input_devices()
            .map_err(|e| AppError::Audio(e.to_string()))?
            .nth(idx)
            .ok_or_else(|| AppError::Audio(format!("Device index {} not found", idx))),
        None => host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("No default input device".into())),
    }
}

/// Shared buffer for audio samples collected by the recording thread.
struct RecordingBuffer {
    samples: Mutex<Vec<f32>>,
    source_sample_rate: u32,
    source_channels: u16,
    is_recording: AtomicBool,
}

/// Thread-safe audio recorder that manages recording on a dedicated thread.
pub struct AudioRecorder {
    buffer: Option<Arc<RecordingBuffer>>,
    recording_thread: Option<thread::JoinHandle<()>>,
}

// Safety: AudioRecorder's thread handle and Arc buffer are Send+Sync.
// The non-Send cpal::Stream lives only inside the recording thread.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            buffer: None,
            recording_thread: None,
        }
    }

    pub fn start_recording(&mut self, device_index: Option<usize>) -> Result<(), AppError> {
        if self.buffer.is_some() {
            return Err(AppError::Audio("Already recording".into()));
        }

        // Probe device config on the main thread to report errors immediately
        let device = get_input_device(device_index)?;
        let config = device
            .default_input_config()
            .map_err(|e| AppError::Audio(e.to_string()))?;

        let source_sample_rate = config.sample_rate().0;
        let source_channels = config.channels();
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();

        let buffer = Arc::new(RecordingBuffer {
            samples: Mutex::new(Vec::new()),
            source_sample_rate,
            source_channels,
            is_recording: AtomicBool::new(true),
        });

        let buf_clone = Arc::clone(&buffer);

        // Spawn a dedicated thread that owns the cpal::Stream
        let handle = thread::spawn(move || {
            let stream = match sample_format {
                SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, &buf_clone),
                SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, &buf_clone),
                SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, &buf_clone),
                _ => {
                    eprintln!("Unsupported sample format: {:?}", sample_format);
                    return;
                }
            };

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to build audio stream: {}", e);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("Failed to play audio stream: {}", e);
                return;
            }

            // Keep thread alive while recording
            while buf_clone.is_recording.load(Ordering::Relaxed) {
                thread::sleep(std::time::Duration::from_millis(50));
            }
            // Stream is dropped here, stopping recording
        });

        self.buffer = Some(buffer);
        self.recording_thread = Some(handle);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<u8>, AppError> {
        let buffer = self
            .buffer
            .take()
            .ok_or_else(|| AppError::Audio("Not recording".into()))?;

        // Signal the recording thread to stop
        buffer.is_recording.store(false, Ordering::Relaxed);

        // Wait for the recording thread to finish
        if let Some(handle) = self.recording_thread.take() {
            let _ = handle.join();
        }

        let samples = buffer.samples.lock().unwrap();
        let mono_samples = to_mono(&samples, buffer.source_channels);
        let resampled = resample(&mono_samples, buffer.source_sample_rate, TARGET_SAMPLE_RATE);

        encode_wav(&resampled, TARGET_SAMPLE_RATE, TARGET_CHANNELS)
    }
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    buffer: &Arc<RecordingBuffer>,
) -> Result<cpal::Stream, AppError>
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let buf = Arc::clone(buffer);
    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let float_samples: Vec<f32> =
                    data.iter().map(|s| cpal::Sample::from_sample(*s)).collect();
                if let Ok(mut guard) = buf.samples.lock() {
                    guard.extend_from_slice(&float_samples);
                }
            },
            |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )
        .map_err(|e| AppError::Audio(e.to_string()))?;

    Ok(stream)
}

fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    samples
        .chunks_exact(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac) + samples[idx + 1] * frac
        } else if idx < samples.len() {
            samples[idx]
        } else {
            0.0
        };
        output.push(sample);
    }

    output
}

fn encode_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, AppError> {
    let mut buffer = Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer =
        hound::WavWriter::new(&mut buffer, spec).map_err(|e| AppError::Audio(e.to_string()))?;

    for &sample in samples {
        let amplitude = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer
            .write_sample(amplitude)
            .map_err(|e| AppError::Audio(e.to_string()))?;
    }

    writer
        .finalize()
        .map_err(|e| AppError::Audio(e.to_string()))?;

    Ok(buffer.into_inner())
}
