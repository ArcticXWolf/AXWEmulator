use std::sync::mpsc;

use web_time::Instant;

use axwemulator_backends_chip8::{Chip8Options, create_chip8_backend};
use axwemulator_backends_simple::create_simple_backend;
use axwemulator_core::{
    backend::Backend,
    error::Error,
    frontend::{
        Frontend,
        error::FrontendError,
        graphics::FrameReceiver,
        input::{ButtonState, InputEvent, InputSender},
        text::TextReceiver,
    },
};
use egui::{ColorImage, Event, TextureHandle, TextureOptions};

use crate::utils;

use super::textlog_view::TextlogView;

pub struct BackendState {
    backend: Backend,
    backend_last_update: Instant,
}

impl BackendState {
    pub fn new(backend_choice: AvailableBackends, frontend: &mut MainView) -> Self {
        let backend = match backend_choice {
            AvailableBackends::Simple => {
                create_simple_backend(frontend).expect("could not create backend")
            }
            AvailableBackends::Chip8 => {
                let rom_data = if let Some(data) = frontend.main_state.rom.clone() {
                    data
                } else {
                    include_bytes!("../../../../roms/chip8/programs/IBM Logo.ch8").to_vec()
                };

                create_chip8_backend(frontend, Chip8Options { rom_data })
                    .expect("could not create backend")
            }
        };
        Self {
            backend,
            backend_last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.backend_last_update.elapsed();
        let _ = self.backend.run_for(elapsed.into());
        self.backend_last_update = Instant::now();
    }
}

pub struct FrameState {
    framebuffer_texture: Option<TextureHandle>,
    frame_reciever: FrameReceiver,
}

impl FrameState {
    pub fn new(reciever: FrameReceiver) -> Self {
        Self {
            framebuffer_texture: None,
            frame_reciever: reciever,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if let Some((_clock, frame)) = self.frame_reciever.latest() {
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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum AvailableBackends {
    #[default]
    Simple,
    Chip8,
}

#[derive(Debug)]
pub enum ViewCommand {
    LoadRomBinary(Vec<u8>),
}

#[derive(Default)]
pub struct MainState {
    combobox_backend_selection: AvailableBackends,
    rom: Option<Vec<u8>>,
}

pub struct MainView {
    backend_state: Option<BackendState>,
    frame_state: Option<FrameState>,
    main_state: MainState,
    sub_views: SubViews,
    input_sender: Option<InputSender>,
    view_command_reciever: mpsc::Receiver<ViewCommand>,
    view_command_sender: mpsc::Sender<ViewCommand>,
}

impl eframe::App for MainView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_main(ctx);
        self.sub_views.update(ctx);

        self.draw_main(ctx);
        self.sub_views.draw(ctx);

        ctx.request_repaint();
    }
}

impl MainView {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, reciever) = mpsc::channel();

        Self {
            backend_state: Default::default(),
            frame_state: Default::default(),
            main_state: Default::default(),
            sub_views: Default::default(),
            input_sender: Default::default(),
            view_command_reciever: reciever,
            view_command_sender: sender,
        }
    }

    pub fn update_main(&mut self, ctx: &egui::Context) {
        if let Ok(cmd) = self.view_command_reciever.try_recv() {
            match cmd {
                ViewCommand::LoadRomBinary(rom) => self.main_state.rom = Some(rom),
            }
        }

        if let Some(backend_state) = self.backend_state.as_mut() {
            backend_state.update();
        }
        if let Some(frame_state) = self.frame_state.as_mut() {
            frame_state.update(ctx);
        }

        self.handle_input(ctx);
    }

    pub fn draw_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.backend_state.is_none() {
                ui.heading("Emulator Selection");

                egui::ComboBox::from_label("Select emulator backend")
                    .selected_text(format!("{:?}", self.main_state.combobox_backend_selection))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.main_state.combobox_backend_selection,
                            AvailableBackends::Simple,
                            "Simple",
                        );
                        ui.selectable_value(
                            &mut self.main_state.combobox_backend_selection,
                            AvailableBackends::Chip8,
                            "Chip8",
                        );
                    });
                if ui.button("Select rom").clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let sender = self.view_command_sender.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Some(handle) = rfd::AsyncFileDialog::new().pick_file().await {
                                let rom = handle.read().await;
                                sender.send(ViewCommand::LoadRomBinary(rom)).unwrap();
                            }
                        });
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let rom = std::fs::read(path).expect("unable to read rom");
                            self.view_command_sender
                                .send(ViewCommand::LoadRomBinary(rom))
                                .unwrap();
                        }
                    }
                }
                if self.main_state.rom.is_some() {
                    ui.label("Rom loaded.");
                }

                if ui.button("Load emulator backend").clicked() {
                    self.start_new_backend(self.main_state.combobox_backend_selection, ctx);
                }
            } else if let Some(frame_state) = &self.frame_state {
                if let Some(framebuffer_texture) = &frame_state.framebuffer_texture {
                    ui.add(egui::Image::new(framebuffer_texture).shrink_to_fit());
                }
            }
        });
    }

    pub fn start_new_backend(&mut self, selected_backend: AvailableBackends, ctx: &egui::Context) {
        self.backend_state = Some(BackendState::new(selected_backend, self));
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if self.input_sender.is_none() {
            return;
        }

        ctx.input(|i| {
            for event in i.raw.events.iter() {
                if let Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    repeat,
                    modifiers: _,
                } = event
                {
                    if *repeat {
                        continue;
                    }
                    let state = if *pressed {
                        ButtonState::Pressed
                    } else {
                        ButtonState::Released
                    };
                    if let Some(key) = utils::translate_egui_key_to_frontend_key(*key) {
                        self.input_sender
                            .as_ref()
                            .unwrap()
                            .add(InputEvent::Keyboard(key, state));
                    }
                }
            }
        });
    }
}

impl Frontend for MainView {
    type Error = Error;

    fn register_text_reciever(
        &mut self,
        reciever: TextReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        self.sub_views.textlog_view = Some(TextlogView::new(reciever));
        Ok(())
    }

    fn register_graphics_reciever(
        &mut self,
        reciever: FrameReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        self.frame_state = Some(FrameState::new(reciever));
        Ok(())
    }

    fn register_input_sender(
        &mut self,
        sender: axwemulator_core::frontend::input::InputSender,
    ) -> Result<(), FrontendError<Self::Error>> {
        self.input_sender = Some(sender);
        Ok(())
    }
}

#[derive(Default)]
pub struct SubViews {
    textlog_view: Option<TextlogView>,
}

impl SubViews {
    pub fn update(&mut self, ctx: &egui::Context) {
        if let Some(view) = self.textlog_view.as_mut() {
            view.update(ctx);
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        if let Some(view) = self.textlog_view.as_mut() {
            view.draw(ctx);
        }
    }
}
