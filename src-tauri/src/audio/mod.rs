use crate::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SizedSample, Stream};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub const WHISPER_SAMPLE_RATE: u32 = 16_000;
pub const MIN_RECORDING_MS: u64 = 250;
const SILENCE_RMS_THRESHOLD: f32 = 0.006;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrophoneDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    pub fn duration(&self) -> Duration {
        if self.sample_rate == 0 || self.channels == 0 {
            return Duration::ZERO;
        }
        let frames = self.samples.len() as f64 / self.channels as f64;
        Duration::from_secs_f64(frames / self.sample_rate as f64)
    }

    pub fn is_too_short(&self) -> bool {
        self.duration() < Duration::from_millis(MIN_RECORDING_MS)
    }

    pub fn is_silent(&self) -> bool {
        if self.samples.is_empty() {
            return true;
        }
        let sum = self
            .samples
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>();
        let rms = (sum / self.samples.len() as f32).sqrt();
        rms < SILENCE_RMS_THRESHOLD
    }

    pub fn to_whisper_mono(&self) -> Vec<f32> {
        let mono = self.to_mono();
        if self.sample_rate == WHISPER_SAMPLE_RATE {
            return mono;
        }
        resample_linear(&mono, self.sample_rate, WHISPER_SAMPLE_RATE)
    }

    fn to_mono(&self) -> Vec<f32> {
        if self.channels <= 1 {
            return self.samples.clone();
        }

        self.samples
            .chunks(self.channels as usize)
            .map(|frame| frame.iter().copied().sum::<f32>() / frame.len() as f32)
            .collect()
    }
}

pub fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() || from_rate == 0 || to_rate == 0 {
        return Vec::new();
    }
    if from_rate == to_rate {
        return input.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for index in 0..output_len {
        let source = index as f64 * ratio;
        let left = source.floor() as usize;
        let right = (left + 1).min(input.len() - 1);
        let fraction = (source - left as f64) as f32;
        let sample = input[left] * (1.0 - fraction) + input[right] * fraction;
        output.push(sample);
    }

    output
}

pub fn list_input_devices() -> AppResult<Vec<MicrophoneDevice>> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|device| device.name().ok());

    let devices = host
        .input_devices()
        .map_err(|err| AppError::Audio(format!("could not enumerate microphones: {err}")))?;

    let mut output = vec![MicrophoneDevice {
        id: String::new(),
        name: "System default microphone".to_string(),
        is_default: true,
    }];

    for device in devices {
        let name = device
            .name()
            .unwrap_or_else(|_| "Unnamed microphone".to_string());
        let is_default = default_name.as_deref() == Some(name.as_str());
        output.push(MicrophoneDevice {
            id: name.clone(),
            name,
            is_default,
        });
    }

    Ok(output)
}

pub struct CpalAudioRecorder;

impl CpalAudioRecorder {
    pub fn start(device_id: Option<&str>) -> AppResult<RecordingSession> {
        let host = cpal::default_host();
        let device = resolve_input_device(&host, device_id)?;
        let supported = device
            .default_input_config()
            .map_err(|err| AppError::Audio(format!("could not read microphone format: {err}")))?;

        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels();
        let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
        let err_fn = |err| eprintln!("[Voxta] audio stream error: {err}");

        let config = supported.config();
        let stream = match supported.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &config, samples.clone(), err_fn),
            SampleFormat::I16 => build_stream::<i16>(&device, &config, samples.clone(), err_fn),
            SampleFormat::U16 => build_stream::<u16>(&device, &config, samples.clone(), err_fn),
            sample_format => Err(AppError::Audio(format!(
                "unsupported microphone sample format: {sample_format:?}"
            ))),
        }?;

        stream
            .play()
            .map_err(|err| AppError::Audio(format!("could not start microphone: {err}")))?;

        Ok(RecordingSession {
            stream: Some(stream),
            samples,
            sample_rate,
            channels,
            started_at: Instant::now(),
        })
    }
}

#[derive(Clone)]
pub struct AudioRecorderHandle {
    sender: Sender<AudioCommand>,
}

enum AudioCommand {
    Start {
        device_id: String,
        reply: Sender<AppResult<()>>,
    },
    Stop {
        reply: Sender<AppResult<AudioBuffer>>,
    },
    Cancel,
}

impl AudioRecorderHandle {
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::channel::<AudioCommand>();
        thread::spawn(move || {
            let mut session: Option<RecordingSession> = None;

            while let Ok(command) = receiver.recv() {
                match command {
                    AudioCommand::Start { device_id, reply } => {
                        if session.is_some() {
                            let _ = reply.send(Err(AppError::Audio(
                                "a recording is already active".to_string(),
                            )));
                            continue;
                        }
                        let result =
                            CpalAudioRecorder::start(Some(&device_id)).map(|next_session| {
                                session = Some(next_session);
                            });
                        let _ = reply.send(result);
                    }
                    AudioCommand::Stop { reply } => {
                        let result = session
                            .take()
                            .map(RecordingSession::stop)
                            .ok_or_else(|| AppError::Audio("no recording is active".to_string()));
                        let _ = reply.send(result);
                    }
                    AudioCommand::Cancel => {
                        session.take();
                    }
                }
            }
        });

        Self { sender }
    }

    pub fn start(&self, device_id: &str) -> AppResult<()> {
        let (reply, response) = mpsc::channel();
        self.sender
            .send(AudioCommand::Start {
                device_id: device_id.to_string(),
                reply,
            })
            .map_err(|err| AppError::Audio(format!("audio worker is unavailable: {err}")))?;
        response
            .recv()
            .map_err(|err| AppError::Audio(format!("audio worker did not respond: {err}")))?
    }

    pub fn stop(&self) -> AppResult<AudioBuffer> {
        let (reply, response) = mpsc::channel();
        self.sender
            .send(AudioCommand::Stop { reply })
            .map_err(|err| AppError::Audio(format!("audio worker is unavailable: {err}")))?;
        response
            .recv()
            .map_err(|err| AppError::Audio(format!("audio worker did not respond: {err}")))?
    }

    pub fn cancel(&self) {
        let _ = self.sender.send(AudioCommand::Cancel);
    }
}

pub struct RecordingSession {
    stream: Option<Stream>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    started_at: Instant,
}

impl RecordingSession {
    pub fn stop(mut self) -> AudioBuffer {
        self.stream.take();
        AudioBuffer {
            sample_rate: self.sample_rate,
            channels: self.channels,
            samples: self.samples.lock().clone(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

fn resolve_input_device(host: &cpal::Host, device_id: Option<&str>) -> AppResult<cpal::Device> {
    let requested = device_id.unwrap_or_default();
    if requested.trim().is_empty() {
        return host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("no default microphone is available".to_string()));
    }

    let devices = host
        .input_devices()
        .map_err(|err| AppError::Audio(format!("could not enumerate microphones: {err}")))?;
    for device in devices {
        if device.name().ok().as_deref() == Some(requested) {
            return Ok(device);
        }
    }

    Err(AppError::Audio(format!(
        "selected microphone is unavailable: {requested}"
    )))
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> AppResult<Stream>
where
    T: Sample + SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut target = samples.lock();
                target.extend(data.iter().map(|sample| f32::from_sample(*sample)));
            },
            err_fn,
            None,
        )
        .map_err(|err| AppError::Audio(format!("could not build microphone stream: {err}")))
}

#[cfg(test)]
mod tests {
    use super::{resample_linear, AudioBuffer, WHISPER_SAMPLE_RATE};

    #[test]
    fn converts_stereo_to_mono() {
        let audio = AudioBuffer {
            sample_rate: WHISPER_SAMPLE_RATE,
            channels: 2,
            samples: vec![1.0, 0.0, 0.5, -0.5],
        };
        assert_eq!(audio.to_whisper_mono(), vec![0.5, 0.0]);
    }

    #[test]
    fn resamples_to_target_rate() {
        let input = vec![0.0, 1.0, 0.0, -1.0];
        let output = resample_linear(&input, 4, 2);
        assert_eq!(output.len(), 2);
        assert!((output[0] - 0.0).abs() < 0.001);
    }

    #[test]
    fn detects_empty_and_silent_recordings() {
        let audio = AudioBuffer {
            sample_rate: WHISPER_SAMPLE_RATE,
            channels: 1,
            samples: vec![0.0; 32000],
        };
        assert!(audio.is_silent());
        assert!(!audio.is_too_short());
    }
}
