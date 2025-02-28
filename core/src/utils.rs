use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

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

    pub fn is_empty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }
}
