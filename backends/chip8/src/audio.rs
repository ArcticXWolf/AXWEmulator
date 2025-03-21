use axwemulator_core::{
    backend::{
        Backend,
        component::{Addressable, Steppable, Transmutable},
    },
    error::Error,
    frontend::audio::AudioSender,
};
use femtos::Duration;

use crate::ST_TIMER;

pub const AUDIO_SAMPLING_RATE: f32 = 48_000.0;
pub const AUDIO_CLOCK_SPEED_NS: u64 = 1_000_000_000 / (AUDIO_SAMPLING_RATE as u64);

pub struct Audio {
    sample_clock: f32,
    audio_sender: AudioSender,
}

impl Audio {
    pub fn new(audio_sender: AudioSender) -> Self {
        Self {
            sample_clock: 0.0,
            audio_sender,
        }
    }
}

impl Steppable for Audio {
    fn step(&mut self, backend: &Backend) -> Result<Duration, Error> {
        let st = backend.get_bus().read_u8(ST_TIMER)?;

        self.sample_clock = (self.sample_clock + 1.0) % AUDIO_SAMPLING_RATE;
        let sample = if st > 0 {
            (self.sample_clock * 440.0 * 2.0 * std::f32::consts::PI / AUDIO_SAMPLING_RATE).sin()
        } else {
            0.0
        };
        self.audio_sender.add(backend.get_current_clock(), sample);

        Ok(Duration::from_nanos(AUDIO_CLOCK_SPEED_NS))
    }
}

impl Transmutable for Audio {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }
}
