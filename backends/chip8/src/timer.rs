use axwemulator_core::{
    backend::{
        Backend,
        component::{Addressable, Steppable, Transmutable},
    },
    error::Error,
};
use femtos::Duration;

use crate::{DT_TIMER, ST_TIMER};

pub const TIMER_CLOCK_SPEED_NS: u64 = 1_000_000_000 / 60;

#[derive(Default)]
pub struct Timer {}

impl Timer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Steppable for Timer {
    fn step(&mut self, backend: &Backend) -> Result<Duration, Error> {
        let dt = backend.get_bus().read_u8(DT_TIMER)?;
        let st = backend.get_bus().read_u8(ST_TIMER)?;

        if dt > 0 {
            backend.get_bus().write_u8(DT_TIMER, dt.saturating_sub(1))?;
        }

        if st > 0 {
            backend.get_bus().write_u8(ST_TIMER, st.saturating_sub(1))?;
        }

        Ok(Duration::from_nanos(TIMER_CLOCK_SPEED_NS))
    }
}

impl Transmutable for Timer {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }
}
