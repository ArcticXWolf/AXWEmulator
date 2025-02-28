use crate::error::{EmulatorErrorKind, Error};

use super::component::{Addressable, Component, MemoryAddress, MemorySize, Transmutable};

#[derive(Default)]
pub struct MemoryBlock {
    read_only: bool,
    data: Vec<u8>,
}

impl From<Vec<u8>> for MemoryBlock {
    fn from(value: Vec<u8>) -> Self {
        MemoryBlock {
            data: value,
            ..Default::default()
        }
    }
}

impl MemoryBlock {
    pub fn set_read_only(&mut self) {
        self.read_only = true;
    }

    pub fn resize(&mut self, size: MemorySize) {
        self.data.resize(size, 0);
    }
}

impl Addressable for MemoryBlock {
    fn size(&self) -> MemorySize {
        self.data.len()
    }

    fn read(&self, address: MemoryAddress, buffer: &mut [u8]) -> Result<(), Error> {
        if address + buffer.len() > self.size() {
            return Err(Error::emulator(
                EmulatorErrorKind::MemoryAccessOutOfBounds,
                format!(
                    "memory block of size {:#010x}, but read {:#010x} - {:#010x}",
                    self.size(),
                    address,
                    address + buffer.len()
                ),
            ));
        }
        buffer.copy_from_slice(&self.data[address..address + buffer.len()]);
        Ok(())
    }

    fn write(&mut self, address: MemoryAddress, buffer: &[u8]) -> Result<(), Error> {
        if self.read_only {
            return Err(Error::emulator(
                EmulatorErrorKind::MemoryAccessReadOnly,
                format!(
                    "memory block of size {:#010x}, request {:#010x} - {:#010x}",
                    self.size(),
                    address,
                    address + buffer.len()
                ),
            ));
        }

        if address + buffer.len() > self.size() {
            return Err(Error::emulator(
                EmulatorErrorKind::MemoryAccessOutOfBounds,
                format!(
                    "memory block of size {:#010x}, but wrote {:#010x} - {:#010x}",
                    self.size(),
                    address,
                    address + buffer.len()
                ),
            ));
        }

        self.data[address..address + buffer.len()].copy_from_slice(buffer);
        Ok(())
    }
}

impl Transmutable for MemoryBlock {
    fn as_addressable(&mut self) -> Option<&mut dyn Addressable> {
        Some(self)
    }
}

#[derive(Clone)]
pub struct BusMount {
    base: MemoryAddress,
    size: MemorySize,
    component: Component,
}

impl BusMount {
    pub fn contains(&self, address: MemoryAddress) -> bool {
        (self.base <= address) && (address < self.base + self.size)
    }
}

#[derive(Clone, Default)]
pub struct Bus {
    mounts: Vec<BusMount>,
}

impl Bus {
    pub fn insert(&mut self, base: MemoryAddress, component: Component) {
        // TODO: Assert this memory space isnt used already
        let size = component.borrow_mut().as_addressable().unwrap().size();
        self.mounts.push(BusMount {
            base,
            size,
            component,
        });
        self.mounts.sort_by_key(|m| m.base);
    }

    pub fn get_component_at(
        &self,
        address: MemoryAddress,
        size: MemorySize,
    ) -> Result<(Component, MemoryAddress), Error> {
        if size > 0 {
            for mount in &self.mounts {
                if mount.contains(address) && mount.contains(address + size - 1) {
                    return Ok((mount.component.clone(), address - mount.base));
                }
            }
        }
        Err(Error::Emulator(
            EmulatorErrorKind::Misc,
            format!(
                "requested address {:#010x} .. {:#010x}, but found no mapped component",
                address,
                address + size
            ),
        ))
    }
}

impl Addressable for Bus {
    fn size(&self) -> MemorySize {
        let last_mount = self.mounts.last().unwrap();
        last_mount.base + last_mount.size
    }

    fn read(&self, address: MemoryAddress, buffer: &mut [u8]) -> Result<(), Error> {
        let (component, relative_address) = self.get_component_at(address, buffer.len())?;
        component
            .borrow_mut()
            .as_addressable()
            .unwrap()
            .read(relative_address, buffer)
    }

    fn write(&mut self, address: MemoryAddress, buffer: &[u8]) -> Result<(), Error> {
        let (component, relative_address) = self.get_component_at(address, buffer.len())?;
        component
            .borrow_mut()
            .as_addressable()
            .unwrap()
            .write(relative_address, buffer)
    }
}
