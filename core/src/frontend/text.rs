use femtos::Instant;

use crate::utils::Ringbuffer;

pub struct TextSender {
    queue: Ringbuffer<String>,
}

impl TextSender {
    pub fn add(&self, clock: Instant, msg: String) {
        self.queue.push_back(clock, msg);
    }
}

pub struct TextReceiver {
    queue: Ringbuffer<String>,
}

impl TextReceiver {
    pub fn pop(&self) -> Option<(Instant, String)> {
        self.queue.pop_front()
    }
    pub fn latest(&self) -> Option<(Instant, String)> {
        self.queue.drain_and_pop_latest()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

pub fn build_text_channel() -> (TextSender, TextReceiver) {
    let sender = TextSender {
        queue: Ringbuffer::new(20),
    };

    let reciever = TextReceiver {
        queue: sender.queue.clone(),
    };

    (sender, reciever)
}
