use std::sync::mpsc;

use crate::app::AppCommand;

use super::emulator::AvailableBackends;

#[derive(Default)]
pub struct SelectionComponent {
    emulator_backend_selection: AvailableBackends,
}

impl SelectionComponent {
    pub fn new() -> Self {
        Self {
            emulator_backend_selection: Default::default(),
        }
    }

    pub fn update(&mut self, _command_sender: &mpsc::Sender<AppCommand>, _ctx: &egui::Context) {}

    pub fn draw(
        &mut self,
        command_sender: &mpsc::Sender<AppCommand>,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Emulator Selection");
        egui::ComboBox::from_label("Select emulator backend")
            .selected_text(format!("{:?}", self.emulator_backend_selection))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.emulator_backend_selection,
                    AvailableBackends::Chip8,
                    "Chip8",
                );
                ui.selectable_value(
                    &mut self.emulator_backend_selection,
                    AvailableBackends::SuperChip,
                    "SuperChip",
                );
            });
        if ui.button("Select rom").clicked() {
            #[cfg(target_arch = "wasm32")]
            {
                let sender = command_sender.clone();
                let selection = self.emulator_backend_selection.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(handle) = rfd::AsyncFileDialog::new().pick_file().await {
                        let rom = handle.read().await;
                        sender
                            .send(AppCommand::InitBackendWithRom(selection, rom))
                            .unwrap();
                    }
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let rom = std::fs::read(path).expect("unable to read rom");
                    command_sender
                        .send(AppCommand::InitBackendWithRom(
                            self.emulator_backend_selection,
                            rom,
                        ))
                        .unwrap();
                }
            }
        }
    }
}
