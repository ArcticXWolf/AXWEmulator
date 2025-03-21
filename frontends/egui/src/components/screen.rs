use std::sync::mpsc;

use axwemulator_core::frontend::graphics::FrameReceiver;
use egui::{ColorImage, TextureHandle, TextureOptions};

use crate::app::AppCommand;

use super::Component;

pub struct ScreenComponent {
    frame_receiver: FrameReceiver,
    framebuffer_texture: Option<TextureHandle>,
}

impl ScreenComponent {
    pub fn new(frame_receiver: FrameReceiver) -> Self {
        Self {
            frame_receiver,
            framebuffer_texture: None,
        }
    }
}

impl Component for ScreenComponent {
    fn update(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        _command_sender: &mpsc::Sender<AppCommand>,
        ctx: &egui::Context,
    ) {
        if let Some((_clock, frame)) = self.frame_receiver.latest() {
            self.framebuffer_texture = Some(ctx.load_texture(
                "screen",
                ColorImage::from_rgba_unmultiplied(
                    [frame.width as _, frame.height as _],
                    &frame.as_rgba_vec(),
                ),
                TextureOptions::NEAREST,
            ));
        }
    }

    fn draw(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        if let Some(framebuffer_texture) = self.framebuffer_texture.as_ref() {
            ui.add(egui::Image::new(framebuffer_texture).shrink_to_fit());
        }
    }
}
