use std::sync::mpsc;

use web_time::Duration;

use crate::app::AppCommand;

use super::Component;

#[derive(Default)]
pub struct MetricsComponent {}

impl MetricsComponent {
    pub fn new() -> Self {
        Self {}
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
        emulator: &super::emulator::EmulatorComponent,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        let frametimes = emulator.get_frametimes().peek_range(..);
        let avg_total = frametimes
            .iter()
            .map(|f| f.total_frametime)
            .sum::<Duration>()
            / frametimes.len() as u32;
        let avg_total_fps = Duration::from_secs(1).as_secs_f64() / avg_total.as_secs_f64();
        let avg_backend = frametimes
            .iter()
            .map(|f| f.emulator_update_frametime)
            .sum::<Duration>()
            / frametimes.len() as u32;

        ui.label(format!(
            "Average Total Frametime: {:06.3}ms ({:.1} FPS)",
            avg_total.as_secs_f64() * 1000.0,
            avg_total_fps
        ));
        ui.label(format!(
            "Average Backend Frametime: {:06.3}ms",
            avg_backend.as_secs_f64() * 1000.0
        ));
    }
}
