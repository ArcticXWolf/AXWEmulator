use std::sync::mpsc;

use axwemulator_core::frontend::input::{ButtonState, InputEvent, InputSender};
use egui::{Event, Key};

use crate::{app::AppCommand, utils};

use super::Component;

pub struct InputComponent {
    input_sender: InputSender,
}

impl InputComponent {
    pub fn new(input_sender: InputSender) -> Self {
        Self { input_sender }
    }
}

impl Component for InputComponent {
    fn update(
        &mut self,
        _emulator: &super::emulator::EmulatorComponent,
        command_sender: &mpsc::Sender<AppCommand>,
        ctx: &egui::Context,
    ) {
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
                        self.input_sender.add(InputEvent::Keyboard(key, state));
                    }
                    if *key == Key::Escape {
                        command_sender.send(AppCommand::QuitBackend).unwrap();
                    }
                }
            }
        });
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
