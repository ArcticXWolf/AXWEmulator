use axwemulator_core::frontend::text::TextReceiver;
use femtos::Instant;

pub struct TextlogState {
    pub loglines: Vec<(Instant, String)>,
    pub text_reciever: TextReceiver,
}

pub struct TextlogView {
    state: TextlogState,
}

// Could be a View-trait?
impl TextlogView {
    pub fn update(&mut self, ctx: &egui::Context) {}

    pub fn draw(&mut self, ctx: &egui::Context) {}
}

impl TextlogView {
    pub fn new(reciever: TextReceiver) -> Self {
        Self {
            state: TextlogState {
                loglines: vec![],
                text_reciever: reciever,
            },
        }
    }
}
