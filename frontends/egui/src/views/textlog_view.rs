use axwemulator_core::{backend::Backend, frontend::text::TextReceiver};
use egui::{ScrollArea, TextStyle};
use femtos::Instant;

pub struct TextlogState {
    pub loglines: Vec<(Instant, String)>,
}

pub struct TextlogView {
    state: TextlogState,
    pub text_reciever: TextReceiver,
}

// Could be a View-trait?
impl TextlogView {
    pub fn update(&mut self, backend: &Backend, ctx: &egui::Context) {
        while let Some(line_with_clock) = self.text_reciever.pop() {
            self.state.loglines.push(line_with_clock);
        }
        if self.state.loglines.len() > 1000 {
            self.state
                .loglines
                .drain(0..(self.state.loglines.len() - 1000));
        }
    }

    pub fn draw(&mut self, backend: &Backend, ctx: &egui::Context, ui: &mut egui::Ui) {
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        ScrollArea::vertical().stick_to_bottom(true).show_rows(
            ui,
            row_height,
            self.state.loglines.len(),
            |ui, row_range| {
                for row in row_range {
                    let (clock, line) = self.state.loglines.get(row).unwrap();
                    ui.label(format!("LOG at {:>25?}: {}", clock, line));
                }
            },
        );
    }
}

impl TextlogView {
    pub fn new(reciever: TextReceiver) -> Self {
        Self {
            state: TextlogState { loglines: vec![] },
            text_reciever: reciever,
        }
    }
}
