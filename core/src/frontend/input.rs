use crate::utils::Ringbuffer;

#[derive(Debug, Clone, Copy)]
pub enum KeyboardEventKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Number0,
    Number1,
    Number2,
    Number3,
    Number4,
    Number5,
    Number6,
    Number7,
    Number8,
    Number9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Keyboard(KeyboardEventKey, ButtonState),
    // controller
    // mouse
    // ...
}

pub struct InputSender {
    queue: Ringbuffer<InputEvent>,
}

impl InputSender {
    pub fn add(&self, input: InputEvent) {
        self.queue.push_back(femtos::Instant::START, input);
    }
}

pub struct InputReceiver {
    queue: Ringbuffer<InputEvent>,
}

impl InputReceiver {
    pub fn pop(&self) -> Option<InputEvent> {
        if let Some((_, ie)) = self.queue.pop_front() {
            Some(ie)
        } else {
            None
        }
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

pub fn build_input_channel() -> (InputSender, InputReceiver) {
    let sender = InputSender {
        queue: Ringbuffer::new(20),
    };

    let reciever = InputReceiver {
        queue: sender.queue.clone(),
    };

    (sender, reciever)
}
