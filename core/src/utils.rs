use std::{
    collections::VecDeque,
    ops::RangeBounds,
    sync::{Arc, Mutex},
};

use femtos::Instant;

#[derive(Clone, Default)]
pub struct Ringbuffer<T>(Arc<Mutex<VecDeque<T>>>, usize);

impl<T: Clone> Ringbuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self(
            Arc::new(Mutex::new(VecDeque::with_capacity(capacity + 1))),
            capacity,
        )
    }

    pub fn push_back(&self, value: T) {
        let mut queue = self.0.lock().unwrap();
        if queue.len() >= self.1 {
            queue.pop_front();
        }
        queue.push_back(value);
    }

    pub fn pop_front(&self) -> Option<T> {
        self.0.lock().unwrap().pop_front()
    }

    pub fn drain_and_pop_latest(&self) -> Option<T> {
        self.0.lock().unwrap().drain(..).last()
    }

    pub fn drain_and_pop_range<R>(&self, range: R) -> Vec<T>
    where
        R: RangeBounds<usize>,
    {
        self.0.lock().unwrap().drain(range).collect::<Vec<T>>()
    }

    pub fn peek_range<R>(&self, range: R) -> Vec<T>
    where
        R: RangeBounds<usize>,
    {
        self.0
            .lock()
            .unwrap()
            .range(range)
            .cloned()
            .collect::<Vec<T>>()
    }

    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    pub fn capacity(&self) -> usize {
        self.1
    }
}

pub type ClockedRingbuffer<T> = Ringbuffer<(Instant, T)>;
