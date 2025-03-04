pub mod component;
pub mod memory;

use std::{
    cell::{RefCell, RefMut},
    collections::{BinaryHeap, HashMap},
    rc::Rc,
};

use component::{Component, MemoryAddress};
use femtos::{Duration, Instant};
use memory::Bus;

use crate::error::Error;

pub struct Backend {
    clock: Instant,
    components: HashMap<String, Component>,
    scheduler_queue: BinaryHeap<SchedulerEvent>,
    bus: Rc<RefCell<Bus>>,
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            clock: Instant::START,
            components: HashMap::new(),
            scheduler_queue: BinaryHeap::new(),
            bus: Rc::new(RefCell::new(Bus::default())),
        }
    }
}

impl Backend {
    pub fn get_bus(&self) -> RefMut<'_, Bus> {
        self.bus.borrow_mut()
    }

    pub fn get_device(&self, name: &str) -> Result<Component, Error> {
        self.components
            .get(name)
            .cloned()
            .ok_or_else(|| Error::new(format!("no component named {}", name)))
    }

    pub fn get_current_clock(&self) -> Instant {
        self.clock
    }

    pub fn add_addressable_component(
        &mut self,
        name: &str,
        address: MemoryAddress,
        component: Component,
    ) {
        self.bus.borrow_mut().insert(address, component.clone());
        self.add_component(name, component);
    }

    pub fn add_component(&mut self, name: &str, component: Component) {
        self.try_queue_component(component.clone());
        self.components.insert(name.to_string(), component);
    }

    pub fn step(&mut self) -> Result<(), Error> {
        let mut next_event = self.scheduler_queue.pop().unwrap();
        self.clock = next_event.clock_cycle;

        let result = match next_event
            .component
            .borrow_mut()
            .as_steppable()
            .unwrap()
            .step(self)
        {
            Ok(next_event_in) => {
                next_event.clock_cycle = self.clock.checked_add(next_event_in).unwrap();
                Ok(())
            }
            Err(err) => Err(err),
        };
        self.queue_event(next_event);
        result
    }

    pub fn run_until(&mut self, clock: Instant) -> Result<(), Error> {
        while self.clock < clock {
            self.step()?;
        }
        Ok(())
    }

    pub fn run_for(&mut self, duration: Duration) -> Result<(), Error> {
        let clock = self.clock + duration;
        self.run_until(clock)
    }

    fn try_queue_component(&mut self, component: Component) {
        if component.borrow_mut().as_steppable().is_some() {
            self.queue_event(SchedulerEvent::new(component));
        }
    }

    fn queue_event(&mut self, event: SchedulerEvent) {
        self.scheduler_queue.push(event);
    }
}

#[derive(PartialEq, Eq)]
struct SchedulerEvent {
    clock_cycle: Instant,
    component: Component,
}

impl SchedulerEvent {
    fn new(component: Component) -> Self {
        Self {
            clock_cycle: Instant::START,
            component,
        }
    }
}

// We flip the ordering on ScheduleEvent, such that scheduler_queue will be a min_heap
impl Ord for SchedulerEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.clock_cycle.cmp(&self.clock_cycle)
    }
}

impl PartialOrd for SchedulerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
