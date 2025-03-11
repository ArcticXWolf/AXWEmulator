use std::sync::mpsc;

use cpal::{
    FromSample, Sample, SizedSample, Stream, StreamError,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use web_time::Instant;

use axwemulator_backends_chip8::{Chip8Options, Platform, create_chip8_backend};
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

use super::{memory_view::MemoryView, textlog_view::TextlogView};

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
            AvailableBackends::Chip8 | AvailableBackends::SuperChip => {
                let rom_data = if let Some(data) = frontend.main_state.rom.clone() {
                    data
                } else {
                    include_bytes!("../../../../roms/chip8/programs/IBM Logo.ch8").to_vec()
                };
                let platform = match backend_choice {
                    AvailableBackends::Chip8 => Platform::Chip8,
                    AvailableBackends::SuperChip => Platform::SuperChip,
                    _ => unreachable!(),
                };

                create_chip8_backend(frontend, Chip8Options { platform, rom_data })
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
        let result = self.backend.run_for(elapsed.into());
        if let Err(error) = result {
            panic!("{}", error);
        }
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
    Simple,
    #[default]
    Chip8,
    SuperChip,
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
    stream: Option<Stream>,
}

impl eframe::App for MainView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_main(ctx);
        if let Some(backend_state) = self.backend_state.as_ref() {
            self.sub_views.update(&backend_state.backend, ctx);
        }

        if let Some(backend_state) = self.backend_state.as_ref() {
            self.sub_views.draw(&backend_state.backend, ctx);
        }
        self.draw_main(ctx);

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
            stream: Default::default(),
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
                        ui.selectable_value(
                            &mut self.main_state.combobox_backend_selection,
                            AvailableBackends::SuperChip,
                            "SuperChip",
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

                if let Some(stream) = self.stream.as_ref() {
                    if ui.button("Play").clicked() {
                        stream.play();
                    }
                    if ui.button("Pause").clicked() {
                        stream.pause();
                    }
                } else {
                    if ui.button("Setup Audio").clicked() {
                        self.setup_audio();
                    }
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
        self.sub_views = SubViews::new();
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

    pub fn setup_audio(&mut self) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output available");
        let config = device.default_output_config().unwrap();
        println!("Default output config: {:?}", config);

        match config.sample_format() {
            cpal::SampleFormat::I8 => self.run::<i8>(&device, &config.into()),
            cpal::SampleFormat::I16 => self.run::<i16>(&device, &config.into()),
            cpal::SampleFormat::I32 => self.run::<i32>(&device, &config.into()),
            // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
            cpal::SampleFormat::I64 => self.run::<i64>(&device, &config.into()),
            cpal::SampleFormat::U8 => self.run::<u8>(&device, &config.into()),
            cpal::SampleFormat::U16 => self.run::<u16>(&device, &config.into()),
            // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
            cpal::SampleFormat::U32 => self.run::<u32>(&device, &config.into()),
            // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
            cpal::SampleFormat::U64 => self.run::<u64>(&device, &config.into()),
            cpal::SampleFormat::F32 => self.run::<f32>(&device, &config.into()),
            cpal::SampleFormat::F64 => self.run::<f64>(&device, &config.into()),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        }
    }

    pub fn run<T>(&mut self, device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: SizedSample + FromSample<f32>,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        // Produce a sinusoid of maximum amplitude.
        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
            None,
        );
        self.stream = Some(stream.unwrap());
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
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
    memory_view: Option<MemoryView>,
}

impl SubViews {
    pub fn new() -> Self {
        Self {
            textlog_view: Default::default(),
            memory_view: Some(MemoryView::new()),
        }
    }

    pub fn update(&mut self, backend: &Backend, ctx: &egui::Context) {
        if let Some(view) = self.textlog_view.as_mut() {
            view.update(backend, ctx);
        }
        if let Some(view) = self.memory_view.as_mut() {
            view.update(backend, ctx);
        }
    }

    pub fn draw(&mut self, backend: &Backend, ctx: &egui::Context) {
        egui::SidePanel::right("subviews")
            .exact_width(350.0)
            .show(ctx, |ui| {
                if let Some(view) = self.textlog_view.as_mut() {
                    view.draw(backend, ctx, ui);
                }
                if let Some(view) = self.memory_view.as_mut() {
                    view.draw(backend, ctx, ui);
                }
            });
    }
}
