use std::{ops::Deref, sync::mpsc};

use axwemulator_core::backend::component::Addressable;
use egui::{RichText, ScrollArea, TextStyle};

use crate::app::AppCommand;

use super::Component;

const BYTES_PER_ROW: usize = 8;

#[derive(Default)]
pub struct MemoryComponent {
    selected_component: Option<String>,
}

impl MemoryComponent {
    pub fn new() -> Self {
        Self {
            selected_component: None,
        }
    }

    pub fn draw_for_component<T>(&self, ui: &mut egui::Ui, addressable: &T)
    where
        T: Addressable + ?Sized,
    {
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        let row_amount = addressable.size() / BYTES_PER_ROW;

        ScrollArea::vertical().show_rows(ui, row_height, row_amount, |ui, row_range| {
            let mut data = [0u8; BYTES_PER_ROW];

            for row in row_range {
                let address = row * BYTES_PER_ROW;

                addressable.read(address, &mut data).unwrap();

                let mut line = format!("{:#010X} | ", address);

                for b in data {
                    line = format!("{}{:02X} ", line, b);
                }

                ui.label(RichText::new(line).monospace());
            }
        });
    }
}

impl Component for MemoryComponent {
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
        egui::ComboBox::from_label("Memory")
            .selected_text(
                self.selected_component
                    .clone()
                    .unwrap_or(String::from("Bus"))
                    .to_string(),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected_component, None, "Bus");
                for (name, component) in emulator.get_backend().get_all_components() {
                    if component.borrow_mut().as_addressable().is_some() {
                        ui.selectable_value(&mut self.selected_component, Some(name.clone()), name);
                    }
                }
            });

        if let Some(component_name) = &self.selected_component {
            if let Ok(component) = emulator.get_backend().get_component(component_name) {
                if let Some(addressable) = component.borrow_mut().as_addressable() {
                    self.draw_for_component(ui, addressable);
                }
            }
        } else if self.selected_component.is_none() {
            self.draw_for_component(ui, emulator.get_backend().get_bus().deref());
        }
    }
}
