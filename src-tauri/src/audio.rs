use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample,
};
use crossbeam_channel::Sender;
use dictation_core::defaults;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapCons, HeapRb,
};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Serialize)]
pub struct Microphone {
    pub id: String,
    pub label: String,
}

pub fn devices() -> Result<Vec<Microphone>, String> {
    let host = cpal::default_host();
    let mut result = vec![Microphone {
        id: defaults::SYSTEM_DEFAULT_DEVICE.into(),
        label: "Follow system default".into(),
    }];
    for device in host
        .input_devices()
        .map_err(|_| "Cannot enumerate microphones.")?
    {
        let name = device
            .name()
            .map_err(|_| "A microphone did not report its name.")?;
        if !result.iter().any(|d| d.id == name) {
            result.push(Microphone {
                id: name.clone(),
                label: name,
            });
        }
    }
    Ok(result)
}

pub struct Capture {
    _stream: cpal::Stream,
    consumer: HeapCons<f32>,
    pub failed: Arc<AtomicBool>,
    resampler: sherpa_onnx::LinearResampler,
    raw: Vec<f32>,
    pub device_name: String,
}

impl Capture {
    pub fn open(choice: &str, ready: Sender<()>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = if choice == defaults::SYSTEM_DEFAULT_DEVICE {
            host.default_input_device()
                .ok_or("No system microphone is available.")?
        } else {
            let matching: Vec<_> = host
                .input_devices()
                .map_err(|_| "Microphone discovery failed.")?
                .filter(|d| d.name().is_ok_and(|name| name == choice))
                .collect();
            if matching.len() != 1 {
                return Err(
                    "The selected microphone is missing or its name is ambiguous. Choose it again."
                        .into(),
                );
            }
            matching
                .into_iter()
                .next()
                .ok_or("Microphone is unavailable.")?
        };
        let name = device.name().map_err(|_| "Microphone name unavailable.")?;
        let format = device
            .default_input_config()
            .map_err(|_| "Microphone access failed. Check Windows microphone privacy settings.")?;
        let config = format.config();
        let rate = config.sample_rate.0 as usize;
        let channels = config.channels as usize;
        let (producer, consumer) =
            HeapRb::<f32>::new(rate * defaults::AUDIO_BUFFER_SECONDS).split();
        let failed = Arc::new(AtomicBool::new(false));
        let stream = match format.sample_format() {
            cpal::SampleFormat::F32 => {
                build::<f32>(&device, &config, producer, channels, ready, failed.clone())
            }
            cpal::SampleFormat::I16 => {
                build::<i16>(&device, &config, producer, channels, ready, failed.clone())
            }
            cpal::SampleFormat::U16 => {
                build::<u16>(&device, &config, producer, channels, ready, failed.clone())
            }
            _ => return Err("This microphone's sample format is unsupported.".into()),
        }?;
        let resampler =
            sherpa_onnx::LinearResampler::create(rate as i32, defaults::SAMPLE_RATE as i32)
                .ok_or("Audio resampler could not be created.")?;
        stream
            .play()
            .map_err(|_| "Microphone could not start. Check permission and device availability.")?;
        Ok(Self {
            _stream: stream,
            consumer,
            failed,
            resampler,
            raw: Vec::with_capacity(rate),
            device_name: name,
        })
    }

    pub fn drain(&mut self) -> Result<Vec<f32>, String> {
        if self.failed.load(Ordering::Acquire) {
            return Err(
                "Microphone disconnected or the audio buffer overflowed. Recording was stopped."
                    .into(),
            );
        }
        self.raw.clear();
        self.raw.extend(self.consumer.pop_iter());
        Ok(self.resampler.resample(&self.raw, false))
    }
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut producer: ringbuf::HeapProd<f32>,
    channels: usize,
    ready: Sender<()>,
    failed: Arc<AtomicBool>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample,
    f32: cpal::FromSample<T>,
{
    let errors = failed.clone();
    let error_ready = ready.clone();
    // The callback only converts/downmixes into a preallocated SPSC ring and signals a bounded queue.
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                for frame in data.chunks_exact(channels) {
                    let mono =
                        frame.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() / channels as f32;
                    if producer.try_push(mono).is_err() {
                        failed.store(true, Ordering::Release);
                        break;
                    }
                }
                let _ = ready.try_send(());
            },
            move |_| {
                errors.store(true, Ordering::Release);
                let _ = error_ready.try_send(());
            },
            None,
        )
        .map_err(|_| {
            "Cannot open the selected microphone. Check permission and device access.".into()
        })
}
