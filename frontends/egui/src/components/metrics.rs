use std::{collections::HashMap, fmt::Display, sync::mpsc};

use axwemulator_core::utils::Ringbuffer;
use web_time::{Duration, Instant};

use crate::app::AppCommand;

use super::Component;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MeasurementType {
    Frametime,
    FullFrametime,
    EmulatorFrametime,
}

impl Display for MeasurementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeasurementType::Frametime => write!(f, "Frametime"),
            MeasurementType::FullFrametime => write!(f, "FullFrametime"),
            MeasurementType::EmulatorFrametime => write!(f, "Emulator"),
        }
    }
}

pub struct Measurement {
    current_start: Instant,
    history: Ringbuffer<Duration>,
}

impl Measurement {
    pub fn new() -> Self {
        Self {
            current_start: Instant::now(),
            history: Ringbuffer::new(200),
        }
    }

    pub fn start_measurement(&mut self) {
        self.current_start = Instant::now();
    }

    pub fn stop_measurement(&mut self) {
        self.history.push_back(self.current_start.elapsed());
    }

    pub fn average(&self) -> Duration {
        self.history.peek_range(..).iter().sum::<Duration>() / self.history.len() as u32
    }

    pub fn min(&self) -> Duration {
        self.history
            .peek_range(..)
            .into_iter()
            .min()
            .unwrap_or_default()
    }

    pub fn max(&self) -> Duration {
        self.history
            .peek_range(..)
            .into_iter()
            .max()
            .unwrap_or_default()
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct MetricsComponent {
    measurements: HashMap<MeasurementType, Measurement>,
}

impl MetricsComponent {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
        }
    }

    pub fn get_measurement(&self, measurement_type: MeasurementType) -> &Measurement {
        &self.measurements[&measurement_type]
    }

    pub fn start(&mut self, measurement_type: MeasurementType) {
        self.measurements
            .entry(measurement_type)
            .or_default()
            .start_measurement();
    }

    pub fn stop(&mut self, measurement_type: MeasurementType) {
        self.measurements
            .entry(measurement_type)
            .or_default()
            .stop_measurement();
    }
}

impl Component for MetricsComponent {
    fn update(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        _command_sender: &mpsc::Sender<AppCommand>,
        _ctx: &egui::Context,
    ) {
    }

    fn draw(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        for (measurement_type, measurement) in &self.measurements {
            ui.label(format!(
                "{:<20}: {:04.2}ms | {:04.2}ms | {:04.2}ms",
                measurement_type,
                measurement.min().as_secs_f32() * 1000.0,
                measurement.average().as_secs_f32() * 1000.0,
                measurement.max().as_secs_f32() * 1000.0
            ));
        }
    }
}
