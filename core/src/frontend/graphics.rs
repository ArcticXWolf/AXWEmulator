use femtos::Instant;

use crate::utils::Ringbuffer;

pub type Pixel = (u8, u8, u8, u8);

#[derive(Clone, Default)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
}

impl Frame {
    pub fn new(dimensions: (usize, usize)) -> Self {
        let data = vec![(0, 0, 0, 255); dimensions.0 * dimensions.1];
        Frame {
            width: dimensions.0,
            height: dimensions.1,
            data: data.to_vec(),
        }
    }

    pub fn as_rgba_vec(&self) -> Vec<u8> {
        let mut result = vec![];

        for pixel in &self.data {
            result.push(pixel.0);
            result.push(pixel.1);
            result.push(pixel.2);
            result.push(pixel.3);
        }

        result
    }
}

pub struct FrameSender {
    queue: Ringbuffer<Frame>,
}

impl FrameSender {
    pub fn add(&self, clock: Instant, frame: Frame) {
        self.queue.push_back(clock, frame);
    }
}

pub struct FrameReceiver {
    max_size: (usize, usize),
    queue: Ringbuffer<Frame>,
}

impl FrameReceiver {
    pub fn max_size(&self) -> (usize, usize) {
        self.max_size
    }

    pub fn latest(&self) -> Option<(Instant, Frame)> {
        self.queue.drain_and_pop_latest()
    }
}

pub fn build_frame_channel(width: usize, height: usize) -> (FrameSender, FrameReceiver) {
    let sender = FrameSender {
        queue: Ringbuffer::new(20),
    };

    let reciever = FrameReceiver {
        max_size: (width, height),
        queue: sender.queue.clone(),
    };

    (sender, reciever)
}
