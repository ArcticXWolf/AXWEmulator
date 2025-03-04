use std::f32::consts::PI;

use axwemulator_core::{
    backend::{
        Backend,
        component::{Component, Steppable, Transmutable},
    },
    error::Error,
    frontend::{
        Frontend,
        graphics::{Frame, FrameSender, build_frame_channel},
        text::{TextSender, build_text_channel},
    },
};
use femtos::Duration;

struct SimpleCpu {
    counter: u64,
    text_sender: TextSender,
    frame_sender: FrameSender,
}

impl Steppable for SimpleCpu {
    fn step(&mut self, backend: &Backend) -> Result<Duration, Error> {
        self.counter += 1;
        self.text_sender.add(
            backend.get_current_clock(),
            format!("Counter: {}", self.counter),
        );

        let frame = Frame {
            width: 100,
            height: 100,
            data: [(
                (((self.counter as f32 * PI / 40.0).sin() + 1.0) * 255.0) as u8,
                ((((self.counter as f32 + 0.5) * PI / 40.0).sin() + 1.0) * 255.0) as u8,
                ((((self.counter as f32 + 1.0) * PI / 40.0).sin() + 1.0) * 255.0) as u8,
                255,
            ); 100 * 100]
                .to_vec(),
        };
        self.frame_sender.add(backend.get_current_clock(), frame);

        Ok(Duration::from_millis(20))
    }
}

impl Transmutable for SimpleCpu {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }
}

pub fn create_simple_backend<F: Frontend>(frontend: &mut F) -> Result<Backend, Error> {
    let mut backend = Backend::default();

    let (text_sender, text_reciever) = build_text_channel();
    let (frame_sender, frame_reciever) = build_frame_channel(100, 100);

    let cpu = SimpleCpu {
        counter: 0,
        text_sender,
        frame_sender,
    };
    backend.add_component("cpu", Component::new(cpu));
    frontend.register_text_reciever(text_reciever)?;
    frontend.register_graphics_reciever(frame_reciever)?;

    Ok(backend)
}
