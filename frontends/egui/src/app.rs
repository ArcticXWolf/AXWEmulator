use std::sync::mpsc;

use axwemulator_core::{error::Error, frontend::Frontend};

use crate::components::{
    Component,
    audio::AudioComponent,
    emulator::{AvailableBackends, EmulatorComponent},
    input::InputComponent,
    inspector::InspectorComponent,
    metrics::{MeasurementType, MetricsComponent},
    screen::ScreenComponent,
    selection::SelectionComponent,
};

#[derive(Debug)]
pub enum AppCommand {
    InitBackendWithRom(AvailableBackends, Vec<u8>),
    QuitBackend,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SidepanelContent {
    Metrics,
    Inspector,
}

pub struct EmulatorApp {
    app_command_receiver: mpsc::Receiver<AppCommand>,
    app_command_sender: mpsc::Sender<AppCommand>,
    sidepanel_selection: SidepanelContent,
    selection: SelectionComponent,
    emulator: Option<EmulatorComponent>,
    screen: Option<ScreenComponent>,
    input: Option<InputComponent>,
    audio: Option<AudioComponent>,
    metrics: Option<MetricsComponent>,
    inspector: Option<InspectorComponent>,
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(metrics) = self.metrics.as_mut() {
            metrics.stop(MeasurementType::FullFrametime);
            metrics.start(MeasurementType::FullFrametime);
            metrics.start(MeasurementType::Frametime);
        }
        self._handle_commands();
        self._update(ctx);
        self._draw(ctx);
        ctx.request_repaint();
        if let Some(metrics) = self.metrics.as_mut() {
            metrics.stop(MeasurementType::Frametime);
        }
    }
}

impl EmulatorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (app_command_sender, app_command_receiver) = mpsc::channel();
        Self {
            app_command_receiver,
            app_command_sender,
            sidepanel_selection: SidepanelContent::Metrics,
            selection: SelectionComponent::new(),
            emulator: None,
            screen: None,
            input: None,
            audio: None,
            metrics: None,
            inspector: None,
        }
    }

    fn _handle_commands(&mut self) {
        if let Ok(cmd) = self.app_command_receiver.try_recv() {
            match cmd {
                AppCommand::InitBackendWithRom(backend_selection, rom_data) => {
                    self.emulator = Some(EmulatorComponent::from_selection(
                        backend_selection,
                        self,
                        &rom_data,
                    ));
                    self.metrics = Some(MetricsComponent::new());
                    self.inspector = Some(InspectorComponent::new());
                }
                AppCommand::QuitBackend => {
                    self.selection = SelectionComponent::new();
                    self.emulator = None;
                    self.screen = None;
                    self.input = None;
                    self.audio = None;
                    self.metrics = None;
                    self.inspector = None;
                }
            }
        }
    }

    fn _update(&mut self, ctx: &egui::Context) {
        if let Some(emulator) = self.emulator.as_mut() {
            if let Some(metrics) = self.metrics.as_mut() {
                metrics.start(MeasurementType::EmulatorFrametime);
            }
            emulator.update();
            if let Some(metrics) = self.metrics.as_mut() {
                metrics.stop(MeasurementType::EmulatorFrametime);
            }

            if let Some(screen) = self.screen.as_mut() {
                screen.update(emulator, &self.app_command_sender, ctx);
            }

            if let Some(input) = self.input.as_mut() {
                input.update(emulator, &self.app_command_sender, ctx);
            }

            if let Some(audio) = self.audio.as_mut() {
                audio.update(emulator, &self.app_command_sender, ctx);
            }

            if let Some(metrics) = self.metrics.as_mut() {
                metrics.update(emulator, &self.app_command_sender, ctx);
            }

            if let Some(inspector) = self.inspector.as_mut() {
                inspector.update(emulator, &self.app_command_sender, ctx);
            }
        } else {
            self.selection.update(&self.app_command_sender, ctx);
        }
    }

    fn _draw(&mut self, ctx: &egui::Context) {
        if let Some(emulator) = self.emulator.as_mut() {
            egui::SidePanel::right("metrics")
                .exact_width(300.0)
                .show(ctx, |ui| {
                    egui::ComboBox::from_label("Sidepanel")
                        .selected_text(format!("{:?}", self.sidepanel_selection))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.sidepanel_selection,
                                SidepanelContent::Metrics,
                                "Metrics",
                            );
                            ui.selectable_value(
                                &mut self.sidepanel_selection,
                                SidepanelContent::Inspector,
                                "Inspector",
                            );
                        });
                    ui.separator();

                    match self.sidepanel_selection {
                        SidepanelContent::Metrics => {
                            if let Some(metrics) = self.metrics.as_mut() {
                                metrics.draw(emulator, ctx, ui);
                            }
                        }
                        SidepanelContent::Inspector => {
                            if let Some(inspector) = self.inspector.as_mut() {
                                inspector.draw(emulator, ctx, ui);
                            }
                        }
                    }
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(emulator) = self.emulator.as_mut() {
                if let Some(screen) = self.screen.as_mut() {
                    screen.draw(emulator, ctx, ui);
                }

                if let Some(input) = self.input.as_mut() {
                    input.draw(emulator, ctx, ui);
                }
                if let Some(audio) = self.audio.as_mut() {
                    audio.draw(emulator, ctx, ui);
                }
            } else {
                self.selection.draw(&self.app_command_sender, ctx, ui);
            }
        });
    }
}

impl Frontend for EmulatorApp {
    type Error = Error;

    fn register_text_receiver(
        &mut self,
        _receiver: axwemulator_core::frontend::text::TextReceiver,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        Ok(())
    }

    fn register_graphics_receiver(
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

    fn register_audio_receiver(
        &mut self,
        audio_receiver: axwemulator_core::frontend::audio::AudioReceiver,
    ) -> Result<(), axwemulator_core::frontend::error::FrontendError<Self::Error>> {
        self.audio = Some(AudioComponent::new(audio_receiver));
        Ok(())
    }
}
