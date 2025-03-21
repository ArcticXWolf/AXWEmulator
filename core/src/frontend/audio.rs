use std::ops::RangeBounds;

use femtos::Instant;

use crate::utils::ClockedRingbuffer;

pub type Sample = f32;

pub struct AudioSender {
    sample_rate: f32,
    queue: ClockedRingbuffer<Sample>,
}

impl AudioSender {
    pub fn add(&self, clock: Instant, sample: Sample) {
        self.queue.push_back((clock, sample));
    }
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }
}

pub struct AudioReceiver {
    sample_rate: f32,
    queue: ClockedRingbuffer<Sample>,
}

impl AudioReceiver {
    pub fn pop(&self) -> Option<(Instant, Sample)> {
        self.queue.pop_front()
    }
    pub fn pop_range<R>(&self, range: R) -> Vec<(Instant, Sample)>
    where
        R: RangeBounds<usize>,
    {
        self.queue.drain_and_pop_range(range)
    }
    pub fn latest(&self) -> Option<(Instant, Sample)> {
        self.queue.drain_and_pop_latest()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

pub fn build_audio_channel(sample_rate: f32, buffer_size: usize) -> (AudioSender, AudioReceiver) {
    let sender = AudioSender {
        sample_rate,
        queue: ClockedRingbuffer::new(buffer_size),
    };

    let receiver = AudioReceiver {
        sample_rate,
        queue: sender.queue.clone(),
    };

    (sender, receiver)
}
