use std::collections::HashMap;

use axwemulator_core::frontend::input::{ButtonState, InputEvent, KeyboardEventKey};

#[derive(PartialEq, Eq, Hash)]
pub enum InputButton {
    Button0,
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
    Button9,
    ButtonA,
    ButtonB,
    ButtonC,
    ButtonD,
    ButtonE,
    ButtonF,
}

impl From<InputButton> for u8 {
    fn from(value: InputButton) -> Self {
        match value {
            InputButton::Button0 => 0,
            InputButton::Button1 => 1,
            InputButton::Button2 => 2,
            InputButton::Button3 => 3,
            InputButton::Button4 => 4,
            InputButton::Button5 => 5,
            InputButton::Button6 => 6,
            InputButton::Button7 => 7,
            InputButton::Button8 => 8,
            InputButton::Button9 => 9,
            InputButton::ButtonA => 10,
            InputButton::ButtonB => 11,
            InputButton::ButtonC => 12,
            InputButton::ButtonD => 13,
            InputButton::ButtonE => 14,
            InputButton::ButtonF => 15,
        }
    }
}

impl TryFrom<u8> for InputButton {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InputButton::Button0),
            1 => Ok(InputButton::Button1),
            2 => Ok(InputButton::Button2),
            3 => Ok(InputButton::Button3),
            4 => Ok(InputButton::Button4),
            5 => Ok(InputButton::Button5),
            6 => Ok(InputButton::Button6),
            7 => Ok(InputButton::Button7),
            8 => Ok(InputButton::Button8),
            9 => Ok(InputButton::Button9),
            10 => Ok(InputButton::ButtonA),
            11 => Ok(InputButton::ButtonB),
            12 => Ok(InputButton::ButtonC),
            13 => Ok(InputButton::ButtonD),
            14 => Ok(InputButton::ButtonE),
            15 => Ok(InputButton::ButtonF),
            _ => Err(()),
        }
    }
}

impl TryFrom<KeyboardEventKey> for InputButton {
    type Error = ();
    fn try_from(value: KeyboardEventKey) -> Result<Self, Self::Error> {
        match value {
            KeyboardEventKey::Number1 => Ok(InputButton::Button1),
            KeyboardEventKey::Number2 => Ok(InputButton::Button2),
            KeyboardEventKey::Number3 => Ok(InputButton::Button3),
            KeyboardEventKey::Number4 => Ok(InputButton::ButtonC),
            KeyboardEventKey::Q => Ok(InputButton::Button4),
            KeyboardEventKey::W => Ok(InputButton::Button5),
            KeyboardEventKey::E => Ok(InputButton::Button6),
            KeyboardEventKey::R => Ok(InputButton::ButtonD),
            KeyboardEventKey::A => Ok(InputButton::Button7),
            KeyboardEventKey::S => Ok(InputButton::Button8),
            KeyboardEventKey::D => Ok(InputButton::Button9),
            KeyboardEventKey::F => Ok(InputButton::ButtonE),
            KeyboardEventKey::Y => Ok(InputButton::ButtonA),
            KeyboardEventKey::X => Ok(InputButton::Button0),
            KeyboardEventKey::C => Ok(InputButton::ButtonB),
            KeyboardEventKey::V => Ok(InputButton::ButtonF),
            _ => Err(()),
        }
    }
}

pub struct KeypadState(HashMap<InputButton, ButtonState>);

impl KeypadState {
    pub fn new() -> Self {
        KeypadState(HashMap::new())
    }

    pub fn parse_input_event(&mut self, event: InputEvent) {
        println!("Parsing input {:?}", event);
        match event {
            InputEvent::Keyboard(keyboard_event_key, button_state) => {
                if let Ok(button) = InputButton::try_from(keyboard_event_key) {
                    *self.0.entry(button).or_insert(ButtonState::Released) = button_state;
                }
            }
        }
    }

    pub fn get_state_for_button(&self, button: InputButton) -> ButtonState {
        *self.0.get(&button).unwrap_or(&ButtonState::Released)
    }
}
