use std::sync::mpsc;

use axwemulator_core::{error::Error, frontend::Frontend};

use crate::components::{
    Component,
    emulator::{AvailableBackends, EmulatorComponent},
    input::InputComponent,
    screen::ScreenComponent,
    selection::SelectionComponent,
};

#[derive(Debug)]
pub enum AppCommand {
    InitBackendWithRom(AvailableBackends, Vec<u8>),
}

pub struct EmulatorApp {
    app_command_reciever: mpsc::Receiver<AppCommand>,
    app_command_sender: mpsc::Sender<AppCommand>,
    selection: SelectionComponent,
    emulator: Option<EmulatorComponent>,
    screen: Option<ScreenComponent>,
    input: Option<InputComponent>,
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self._handle_commands();
        self._update(ctx);
        self._draw(ctx);
        ctx.request_repaint();
    }
}

impl EmulatorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (app_command_sender, app_command_reciever) = mpsc::channel();
        Self {
            app_command_reciever,
            app_command_sender,
            selection: SelectionComponent::new(),
            emulator: None,
            screen: None,
            input: None,
        }
    }

    fn _handle_commands(&mut self) {
        if let Ok(cmd) = self.app_command_reciever.try_recv() {
            match cmd {
                AppCommand::InitBackendWithRom(backend_selection, rom_data) => {
                    self.emulator = Some(EmulatorComponent::from_selection(
                        backend_selection,
                        self,
                        &rom_data,
                    ));
                }
                _ => {}
            }
        }
    }

    fn _update(&mut self, ctx: &egui::Context) {
        if let Some(emulator) = self.emulator.as_mut() {
            emulator.update();

            if let Some(screen) = self.screen.as_mut() {
                screen.update(emulator, ctx);
            }

            if let Some(input) = self.input.as_mut() {
                input.update(emulator, ctx);
            }
        } else {
            self.selection.update(&self.app_command_sender, ctx);
        }
    }

    fn _draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(emulator) = self.emulator.as_mut() {
                if let Some(screen) = self.screen.as_mut() {
                    screen.draw(emulator, ctx, ui);
                }

                if let Some(input) = self.input.as_mut() {
                    input.draw(emulator, ctx, ui);
                }
            } else {
                self.selection.draw(&self.app_command_sender, ctx, ui);
            }
        });
    }
}

impl Frontend for EmulatorApp {
    type Error = Error;

    fn register_text_reciever(
        &mut self,
        _reciever: axwemulator_core::frontend::text::TextReceiver,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        Ok(())
    }

    fn register_graphics_reciever(
        &mut self,
        frame_receiver: axwemulator_core::frontend::graphics::FrameReceiver,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        self.screen = Some(ScreenComponent::new(frame_receiver));
        Ok(())
    }

    fn register_input_sender(
        &mut self,
        input_sender: axwemulator_core::frontend::input::InputSender,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        self.input = Some(InputComponent::new(input_sender));
        Ok(())
    }
}
