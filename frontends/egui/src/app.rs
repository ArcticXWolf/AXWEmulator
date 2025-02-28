use std::{sync::Arc, time::Instant};

use axwemulator_backends_chip8::{Chip8Options, create_chip8_backend};
use axwemulator_core::{
    backend::Backend,
    error::Error,
    frontend::{Frontend, graphics::FrameReceiver, text::TextReceiver},
};
use egui::{Color32, ColorImage, ImageData, TextureHandle, TextureOptions};

pub struct EmulatorFrontend {
    last_update: Instant,
    output: String,
    backend: Option<Backend>,
    framebuffer_dimensions: (usize, usize),
    framebuffer_texture: Option<TextureHandle>,
    text_reciever: Option<TextReceiver>,
    frame_reciever: Option<FrameReceiver>,
}

impl Default for EmulatorFrontend {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            output: String::new(),
            backend: Default::default(),
            framebuffer_dimensions: Default::default(),
            framebuffer_texture: Default::default(),
            text_reciever: Default::default(),
            frame_reciever: Default::default(),
        }
    }
}

impl Frontend for EmulatorFrontend {
    type Error = Error;

    fn register_text_reciever(
        &mut self,
        reciever: TextReceiver,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        self.text_reciever = Some(reciever);
        Ok(())
    }

    fn register_graphics_reciever(
        &mut self,
        reciever: FrameReceiver,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        self.frame_reciever = Some(reciever);
        Ok(())
    }
}

impl EmulatorFrontend {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut frontend: EmulatorFrontend = Default::default();

        let rom_data = include_bytes!("../../../roms/chip8/programs/IBM Logo.ch8");
        // let rom_data =
        //     include_bytes!("../../../roms/chip8/demos/Particle Demo [zeroZshadow, 2008].ch8");

        let backend = create_chip8_backend(
            &mut frontend,
            Chip8Options {
                rom_data: rom_data.to_vec(),
            },
        )
        .expect("could not create backend");
        frontend.backend = Some(backend);

        if frontend.frame_reciever.is_some() {
            frontend.framebuffer_dimensions = frontend.frame_reciever.as_ref().unwrap().max_size();
            frontend.framebuffer_texture = {
                let blank = Arc::new(ColorImage {
                    size: [
                        frontend.framebuffer_dimensions.0 as _,
                        frontend.framebuffer_dimensions.1 as _,
                    ],
                    pixels: vec![
                        Color32::default();
                        (frontend.framebuffer_dimensions.0 * frontend.framebuffer_dimensions.1)
                            as _
                    ],
                });
                let blank = ImageData::Color(blank);
                Some(
                    cc.egui_ctx
                        .load_texture("framebuffer", blank, TextureOptions::NEAREST),
                )
            };
        }

        frontend
    }
}

impl eframe::App for EmulatorFrontend {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let clock = femtos::Instant::START + femtos::Duration::from(self.last_update.elapsed());
        let _ = self.backend.as_mut().unwrap().run_until(clock);
        if self.text_reciever.is_some() {
            while !self.text_reciever.as_ref().unwrap().is_empty() {
                self.output = format!(
                    "{}\n{}",
                    self.output,
                    self.text_reciever.as_mut().unwrap().pop().unwrap()
                );
            }
        }

        if self.frame_reciever.is_some() {
            if let Some(frame) = self.frame_reciever.as_ref().unwrap().latest() {
                self.framebuffer_texture = Some(ctx.load_texture(
                    "test",
                    ColorImage::from_rgba_unmultiplied(
                        [frame.width as _, frame.height as _],
                        &frame.as_rgba_vec(),
                    ),
                    TextureOptions::NEAREST,
                ));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Emulator");

            if self.frame_reciever.is_some() {
                ui.add(
                    egui::Image::new(self.framebuffer_texture.as_ref().unwrap()).fit_to_exact_size(
                        egui::vec2(
                            self.framebuffer_dimensions.0 as f32 * 8.0,
                            self.framebuffer_dimensions.1 as f32 * 8.0,
                        ),
                    ),
                );
            }
            ui.label(self.output.clone());
        });

        ctx.request_repaint();
    }
}
