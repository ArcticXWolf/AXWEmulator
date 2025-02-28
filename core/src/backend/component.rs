use std::{
    cell::{BorrowMutError, RefCell, RefMut},
    rc::Rc,
    sync::atomic::AtomicUsize,
};

use femtos::Duration;

use crate::{backend::Backend, error::Error};

pub type MemoryAddress = usize;
pub type MemorySize = MemoryAddress;

pub trait Addressable {
    fn size(&self) -> MemorySize;
    fn read(&self, address: MemoryAddress, buffer: &mut [u8]) -> Result<(), Error>;
    fn write(&mut self, address: MemoryAddress, buffer: &[u8]) -> Result<(), Error>;

    fn read_u8(&self, address: MemoryAddress) -> Result<u8, Error> {
        let mut buffer: [u8; 1] = Default::default();
        self.read(address, &mut buffer)?;
        Ok(buffer[0])
    }
    fn read_u16_le(&self, address: MemoryAddress) -> Result<u16, Error> {
        let mut buffer: [u8; 2] = Default::default();
        self.read(address, &mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }
    fn read_u16_be(&self, address: MemoryAddress) -> Result<u16, Error> {
        let mut buffer: [u8; 2] = Default::default();
        self.read(address, &mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    fn write_u8(&mut self, address: MemoryAddress, value: u8) -> Result<(), Error> {
        self.write(address, &[value])
    }
    fn write_u16_le(&mut self, address: MemoryAddress, value: u16) -> Result<(), Error> {
        self.write(address, &value.to_le_bytes())
    }
    fn write_u16_be(&mut self, address: MemoryAddress, value: u16) -> Result<(), Error> {
        self.write(address, &value.to_be_bytes())
    }
}

pub trait Steppable {
    fn step(&mut self, backend: &Backend) -> Result<Duration, Error>;
}

pub trait Transmutable {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        None
    }
    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        None
    }
}

type TransmutableBox = Rc<RefCell<Box<dyn Transmutable>>>;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(usize);

impl Default for ComponentId {
    fn default() -> Self {
        let next = NEXT_ID.load(std::sync::atomic::Ordering::Acquire);
        NEXT_ID.store(next + 1, std::sync::atomic::Ordering::Release);
        Self(next)
    }
}

#[derive(Clone)]
pub struct Component(ComponentId, TransmutableBox);

impl Component {
    pub fn new<T>(implementation: T) -> Self
    where
        T: Transmutable + 'static,
    {
        Self(
            ComponentId::default(),
            Rc::new(RefCell::new(Box::new(implementation))),
        )
    }

    pub fn id(&self) -> ComponentId {
        self.0
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Box<dyn Transmutable>> {
        self.1.borrow_mut()
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, Box<dyn Transmutable>>, BorrowMutError> {
        self.1.try_borrow_mut()
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Component {}
