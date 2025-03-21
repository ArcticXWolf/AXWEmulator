use std::{fmt::Debug, sync::mpsc};

use axwemulator_core::{frontend::audio::AudioReceiver, utils::Ringbuffer};
use cpal::{
    FromSample, Sample, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

use crate::app::AppCommand;

use super::Component;

const CHUNK_SIZE: usize = 1024;
const TARGET: usize = 2 * CHUNK_SIZE;
const MOVING_AVERAGE_RATIO: f64 = 0.05;

pub struct AudioComponent {
    audio_receiver: AudioReceiver,
    input_sample_rate: f64,
    resampler: SincFixedIn<f32>,
    output_buffer: Ringbuffer<f32>,
    output_sample_rate: f64,
    output_stream: Option<Stream>,
    output_buffer_len_average: usize,
    output_buffer_len_average_history: Ringbuffer<usize>,
}

impl AudioComponent {
    pub fn new(audio_receiver: AudioReceiver) -> Self {
        let params = SincInterpolationParameters {
            sinc_len: 64,
            f_cutoff: 0.91,
            oversampling_factor: 1024,
            interpolation: SincInterpolationType::Linear,
            window: WindowFunction::Hann2,
        };

        let resampler = SincFixedIn::<f32>::new(
            48000.0 / (audio_receiver.sample_rate() as f64),
            2.0,
            params,
            CHUNK_SIZE,
            1,
        )
        .unwrap();

        let mut result = Self {
            input_sample_rate: audio_receiver.sample_rate() as f64,
            audio_receiver,
            resampler,
            output_buffer: Ringbuffer::new(5000),
            output_buffer_len_average: 0,
            output_buffer_len_average_history: Ringbuffer::new(60),
            output_sample_rate: 48000.0,
            output_stream: None,
        };

        result.init();

        result
    }

    pub fn init(&mut self) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to get output device");
        let config = device
            .default_output_config()
            .expect("failed to get output config");
        let channels = config.channels();
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };
        let output_buffer = self.output_buffer.clone();

        self.output_sample_rate = config.sample_rate().0 as f64 * 1.02;
        self.output_stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config.into(),
                    move |data, _: &_| write_data::<f32>(data, &output_buffer, channels as usize),
                    err_fn,
                    None,
                )
                .ok(),
            _ => unimplemented!("unimplemented sample format"),
        };
        self.output_stream.as_ref().unwrap().play().unwrap();
    }

    pub fn recalculate_resampler_ratio(&mut self) {
        // slope via regression
        let (mut sx, mut sy, mut sxx, mut sxy) = (0, 0, 0, 0);
        for (idx, avg) in self
            .output_buffer_len_average_history
            .peek_range(..)
            .iter()
            .enumerate()
        {
            sx += idx;
            sy += avg;
            sxx += idx * idx;
            sxy += idx * avg;
        }
        let n = self.output_buffer_len_average_history.len();
        let num = (n * sxy) as f64 - (sx * sy) as f64;
        let den = (n * sxx) as f64 - (sx * sx) as f64;
        let slope: f64 = if den == 0.0 { 0.0 } else { num / den };

        let difference = self.output_buffer_len_average as f64 - TARGET as f64;
        let direction = if difference == 0.0 {
            0.0
        } else {
            difference / difference.abs()
        };

        let mut adjustment = 0.0;

        if direction * slope < -1.0 {
            adjustment = slope.abs() / 4.0;
            if adjustment > 1.0 {
                adjustment = 1.0;
            }
        } else if direction * slope > 0.0 || self.output_buffer_len_average == 0 {
            let skew = (difference.abs() / 400.0) * 10.0;
            adjustment = (slope.abs() + skew) / -2.0;
            if adjustment < -2.0 {
                adjustment = -2.0;
            }
        }

        adjustment *= direction;
        self.output_sample_rate += adjustment;

        self.resampler
            .set_resample_ratio(self.output_sample_rate / self.input_sample_rate, false)
            .unwrap();
    }
}

fn write_data<T>(output: &mut [T], input: &Ringbuffer<f32>, channels: usize)
where
    T: Sample + FromSample<f32> + Debug,
{
    for frame in output.chunks_mut(channels) {
        let received = input.pop_front().unwrap_or(0.0);
        let value: T = T::from_sample(received);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

impl Component for AudioComponent {
    fn update(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        _command_sender: &mpsc::Sender<AppCommand>,
        _ctx: &egui::Context,
    ) {
        // pull samples
        while self.audio_receiver.len() > CHUNK_SIZE {
            let samples = self
                .audio_receiver
                .pop_range(..CHUNK_SIZE)
                .iter()
                .map(|s| s.1)
                .collect::<Vec<f32>>();

            // convert to target sample rate
            let resampled = self.resampler.process(&[samples], None).unwrap();

            for s in resampled.first().unwrap() {
                self.output_buffer.push_back(*s);
            }
        }

        self.output_buffer_len_average =
            ((self.output_buffer_len_average as f64) * (1.0 - MOVING_AVERAGE_RATIO)
                + self.output_buffer.len() as f64 * MOVING_AVERAGE_RATIO) as usize;
        self.output_buffer_len_average_history
            .push_back(self.output_buffer_len_average);

        self.recalculate_resampler_ratio();
    }

    fn draw(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        _ctx: &egui::Context,
        _ui: &mut egui::Ui,
    ) {
        // nothing to draw
    }
}
