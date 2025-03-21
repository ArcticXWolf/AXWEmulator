use femtos::Instant;

use crate::utils::ClockedRingbuffer;

pub struct TextSender {
    queue: ClockedRingbuffer<String>,
}

impl TextSender {
    pub fn add(&self, clock: Instant, msg: String) {
        self.queue.push_back((clock, msg));
    }
}

pub struct TextReceiver {
    queue: ClockedRingbuffer<String>,
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
        queue: ClockedRingbuffer::new(20),
    };

    let receiver = TextReceiver {
        queue: sender.queue.clone(),
    };

    (sender, receiver)
}
