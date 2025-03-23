use std::sync::mpsc;

use emulator::EmulatorComponent;

use crate::app::AppCommand;

pub mod audio;
pub mod emulator;
pub mod input;
pub mod inspector;
pub mod metrics;
pub mod screen;
pub mod selection;

pub trait Component {
    fn update(
        &mut self,
        emulator: &EmulatorComponent,
        command_sender: &mpsc::Sender<AppCommand>,
        ctx: &egui::Context,
    );
    fn draw(&mut self, emulator: &EmulatorComponent, ctx: &egui::Context, ui: &mut egui::Ui);
}
