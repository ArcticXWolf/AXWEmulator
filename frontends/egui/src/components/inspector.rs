use std::sync::mpsc;

use crate::app::AppCommand;

use super::Component;

#[derive(Default)]
pub struct InspectorComponent {
    selected_component: String,
}

impl InspectorComponent {
    pub fn new() -> Self {
        Self {
            selected_component: "".to_string(),
        }
    }
}

impl Component for InspectorComponent {
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
        egui::ComboBox::from_label("Inspector")
            .selected_text(format!("{:?}", self.selected_component))
            .show_ui(ui, |ui| {
                for (name, component) in emulator.get_backend().get_all_components() {
                    if component.borrow_mut().as_inspectable().is_some() {
                        ui.selectable_value(&mut self.selected_component, name.clone(), name);
                    }
                }
            });

        if let Ok(component) = emulator
            .get_backend()
            .get_component(&self.selected_component)
        {
            if let Some(inspectable) = component.borrow_mut().as_inspectable() {
                let lines = inspectable.inspect();
                for line in lines {
                    ui.label(line);
                }
            }
        }
    }
}
