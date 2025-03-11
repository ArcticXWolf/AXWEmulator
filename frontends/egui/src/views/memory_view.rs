use axwemulator_core::{
    backend::{Backend, component::Addressable},
    frontend::text::TextReceiver,
};
use egui::{RichText, ScrollArea, TextStyle};
use femtos::Instant;

const BYTES_PER_ROW: usize = 8;

pub struct MemoryView {}

// Could be a View-trait?
impl MemoryView {
    pub fn update(&mut self, backend: &Backend, ctx: &egui::Context) {}

    pub fn draw(&mut self, backend: &Backend, ctx: &egui::Context, ui: &mut egui::Ui) {
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        let row_amount = backend.get_bus().size() / BYTES_PER_ROW;
        ScrollArea::vertical().show_rows(ui, row_height, row_amount, |ui, row_range| {
            let mut data = [0u8; BYTES_PER_ROW];
            for row in row_range {
                let address = row * BYTES_PER_ROW;
                backend.get_bus().read(address, &mut data);
                let mut line = format!("{:#010X} | ", address);
                for b in data {
                    line = format!("{}{:02X} ", line, b);
                }
                ui.label(RichText::new(line).family(egui::FontFamily::Monospace));
            }
        });
    }
}

impl MemoryView {
    pub fn new() -> Self {
        Self {}
    }
}
