use std::fmt::Display;

use axwemulator_core::{
    backend::{
        Backend,
        component::{Addressable, MemoryAddress, Steppable, Transmutable},
    },
    error::Error,
    frontend::graphics::{Frame, FrameSender},
};
use femtos::Duration;

pub const CLOCK_SPEED_NS: u64 = 1_000_000_000 / 700;
pub const FRAME_DIMENSIONS: (usize, usize) = (64, 32);

pub struct CpuState {
    v: [u8; 16],
    i: u16,
    dt: u8,
    st: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    paused: bool,
    quirks_load: bool,
    quirks_shift: bool,
    quirks_jump: bool,
    frame_buffer: [bool; FRAME_DIMENSIONS.0 * FRAME_DIMENSIONS.1],
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            v: Default::default(),
            i: Default::default(),
            dt: Default::default(),
            st: Default::default(),
            pc: Default::default(),
            sp: Default::default(),
            stack: Default::default(),
            paused: false,
            quirks_load: Default::default(),
            quirks_shift: Default::default(),
            quirks_jump: Default::default(),
            frame_buffer: [false; FRAME_DIMENSIONS.0 * FRAME_DIMENSIONS.1],
        }
    }
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            pc: 0x200,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Cpu {
    state: CpuState,
    frame_sender: Option<FrameSender>,
}

impl Cpu {
    pub fn new(frame_sender: FrameSender) -> Self {
        Self {
            state: CpuState::new(),
            frame_sender: Some(frame_sender),
        }
    }

    fn send_frame(&self) {
        if self.frame_sender.is_none() {
            return;
        }

        let mut frame = Frame::new(FRAME_DIMENSIONS);

        for y in 0..frame.height {
            for x in 0..frame.width {
                let index = y * frame.width + x;
                if self.state.frame_buffer[index] {
                    frame.data[index] = (255, 255, 255, 255);
                }
            }
        }

        self.frame_sender.as_ref().unwrap().add(frame);
    }
}

impl Steppable for Cpu {
    fn step(&mut self, backend: &Backend) -> Result<Duration, Error> {
        if !self.state.paused {
            // fetch
            let opcode = backend
                .get_bus()
                .read_u16_be(self.state.pc as MemoryAddress)?;
            self.state.pc += 2;

            // decode
            let instruction = Instruction::from(opcode);

            // execute
            instruction.execute(self, backend)?;
        }

        Ok(Duration::from_nanos(CLOCK_SPEED_NS))
    }
}

impl Transmutable for Cpu {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }
}

pub enum Instruction {
    // 0XXX
    Sys(MemoryAddress),
    Cls,
    Return,
    // 1XXX
    Jump(MemoryAddress),
    // 2XXX
    Call(MemoryAddress),
    // 3XXX
    SkipIfVImmediate(usize, u8),
    // 4XXX
    SkipIfNotVImmediate(usize, u8),
    // 5XXX
    SkipIfCmp(usize, usize),
    // 6XXX
    LoadVImmediate(usize, u8),
    // 7XXX
    AddVImmediate(usize, u8),
    // 8XXX
    LoadV(usize, usize),
    Or(usize, usize),
    And(usize, usize),
    Xor(usize, usize),
    Add(usize, usize),
    Sub(usize, usize),
    ShiftRight(usize, usize),
    SubN(usize, usize),
    ShiftLeft(usize, usize),
    // 9XXX
    SkipIfNotCmp(usize, usize),
    // AXXX
    LoadIImmediate(MemoryAddress),
    // BXXX
    JumpV0(MemoryAddress),
    // CXXX
    Random(usize, u8),
    // DXXX
    Draw(usize, usize, usize),
    // EXXX
    SkipIfKey(usize),
    SkipIfNotKey(usize),
    // FXXX
    LoadVDT(usize),
    WaitAndLoadKeypress(usize),
    LoadDTV(usize),
    LoadSTV(usize),
    AddIV(usize),
    LoadFontV(usize),
    StoreBCDV(usize),
    StoreAllV(usize),
    LoadAllV(usize),

    Unknown(u16),
}

impl From<u16> for Instruction {
    fn from(value: u16) -> Self {
        match value >> 12 {
            0x0 => match value {
                0x00E0 => Self::Cls,
                0x00EE => Self::Return,
                _ => Self::Sys((value & 0x0FFF) as MemoryAddress),
            },
            0x1 => Self::Jump((value & 0x0FFF) as MemoryAddress),
            0x2 => Self::Call((value & 0x0FFF) as MemoryAddress),
            0x3 => Self::SkipIfVImmediate(((value & 0x0F00) >> 8) as usize, (value & 0x00FF) as u8),
            0x4 => {
                Self::SkipIfNotVImmediate(((value & 0x0F00) >> 8) as usize, (value & 0x00FF) as u8)
            }
            0x5 => Self::SkipIfCmp(
                ((value & 0x0F00) >> 8) as usize,
                ((value & 0x00F0) >> 4) as usize,
            ),
            0x6 => Self::LoadVImmediate(((value & 0x0F00) >> 8) as usize, (value & 0x00FF) as u8),
            0x7 => Self::AddVImmediate(((value & 0x0F00) >> 8) as usize, (value & 0x00FF) as u8),
            0x8 => match value & 0xF {
                0x0 => Self::LoadV(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x1 => Self::Or(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x2 => Self::And(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x3 => Self::Xor(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x4 => Self::Add(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x5 => Self::Sub(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x6 => Self::ShiftRight(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0x7 => Self::SubN(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                0xE => Self::ShiftLeft(
                    ((value & 0x0F00) >> 8) as usize,
                    ((value & 0x00F0) >> 4) as usize,
                ),
                _ => Self::Unknown(value),
            },
            0x9 => Self::SkipIfNotCmp(
                ((value & 0x0F00) >> 8) as usize,
                ((value & 0x00F0) >> 4) as usize,
            ),
            0xA => Self::LoadIImmediate((value & 0x0FFF) as MemoryAddress),
            0xB => Self::JumpV0((value & 0x0FFF) as MemoryAddress),
            0xC => Self::Random(((value & 0x0F00) >> 8) as usize, (value & 0x00FF) as u8),
            0xD => Self::Draw(
                ((value & 0x0F00) >> 8) as usize,
                ((value & 0x00F0) >> 4) as usize,
                (value & 0x000F) as usize,
            ),
            0xE => match value & 0xFF {
                0x9E => Self::SkipIfKey(((value & 0x0F00) >> 8) as usize),
                0xA1 => Self::SkipIfNotKey(((value & 0x0F00) >> 8) as usize),
                _ => Self::Unknown(value),
            },
            0xF => match value & 0xFF {
                0x07 => Self::LoadVDT(((value & 0x0F00) >> 8) as usize),
                0x0A => Self::WaitAndLoadKeypress(((value & 0x0F00) >> 8) as usize),
                0x15 => Self::LoadDTV(((value & 0x0F00) >> 8) as usize),
                0x18 => Self::LoadSTV(((value & 0x0F00) >> 8) as usize),
                0x1E => Self::AddIV(((value & 0x0F00) >> 8) as usize),
                0x29 => Self::LoadFontV(((value & 0x0F00) >> 8) as usize),
                0x33 => Self::StoreBCDV(((value & 0x0F00) >> 8) as usize),
                0x55 => Self::StoreAllV(((value & 0x0F00) >> 8) as usize),
                0x65 => Self::LoadAllV(((value & 0x0F00) >> 8) as usize),
                _ => Self::Unknown(value),
            },
            _ => Self::Unknown(value),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Sys(x) => write!(f, "SYS[{:#03x}]", x),
            Instruction::Cls => write!(f, "CLS"),
            Instruction::Return => write!(f, "RET"),
            Instruction::Jump(x) => write!(f, "JMP[{:#03x}]", x),
            Instruction::Call(x) => write!(f, "CAL[{:#03x}]", x),
            Instruction::SkipIfVImmediate(x, y) => write!(f, "SKI[{:#03x},{:#03x}]", x, y),
            Instruction::SkipIfNotVImmediate(x, y) => write!(f, "SNI[{:#03x},{:#03x}]", x, y),
            Instruction::SkipIfCmp(x, y) => write!(f, "SKC[{:#03x},{:#03x}]", x, y),
            Instruction::LoadVImmediate(x, y) => write!(f, "LVI[{:#03x},{:#03x}]", x, y),
            Instruction::AddVImmediate(x, y) => write!(f, "AVI[{:#03x},{:#03x}]", x, y),
            Instruction::LoadV(x, y) => write!(f, "LDV[{:#03x},{:#03x}]", x, y),
            Instruction::Or(x, y) => write!(f, "ORV[{:#03x},{:#03x}]", x, y),
            Instruction::And(x, y) => write!(f, "AND[{:#03x},{:#03x}]", x, y),
            Instruction::Xor(x, y) => write!(f, "XOR[{:#03x},{:#03x}]", x, y),
            Instruction::Add(x, y) => write!(f, "ADD[{:#03x},{:#03x}]", x, y),
            Instruction::Sub(x, y) => write!(f, "SUB[{:#03x},{:#03x}]", x, y),
            Instruction::ShiftRight(x, y) => write!(f, "SHR[{:#03x},{:#03x}]", x, y),
            Instruction::SubN(x, y) => write!(f, "SUN[{:#03x},{:#03x}]", x, y),
            Instruction::ShiftLeft(x, y) => write!(f, "SHL[{:#03x},{:#03x}]", x, y),
            Instruction::SkipIfNotCmp(x, y) => write!(f, "SKC[{:#03x},{:#03x}]", x, y),
            Instruction::LoadIImmediate(x) => write!(f, "LDI[{:#03x}]", x),
            Instruction::JumpV0(x) => write!(f, "JPV[{:#03x}]", x),
            Instruction::Random(x, y) => write!(f, "RND[{:#03x},{:#03x}]", x, y),
            Instruction::Draw(x, y, z) => write!(f, "DRW[{:#03x},{:#03x},{:#03x}]", x, y, z),
            Instruction::SkipIfKey(x) => write!(f, "SKP[{:#03x}]", x),
            Instruction::SkipIfNotKey(x) => write!(f, "SNP[{:#03x}]", x),
            Instruction::LoadVDT(x) => write!(f, "LVD[{:#03x}]", x),
            Instruction::WaitAndLoadKeypress(x) => write!(f, "LVK[{:#03x}]", x),
            Instruction::LoadDTV(x) => write!(f, "LDV[{:#03x}]", x),
            Instruction::LoadSTV(x) => write!(f, "LSV[{:#03x}]", x),
            Instruction::AddIV(x) => write!(f, "AIV[{:#03x}]", x),
            Instruction::LoadFontV(x) => write!(f, "LFV[{:#03x}]", x),
            Instruction::StoreBCDV(x) => write!(f, "LBV[{:#03x}]", x),
            Instruction::StoreAllV(x) => write!(f, "LIA[{:#03x}]", x),
            Instruction::LoadAllV(x) => write!(f, "LAI[{:#03x}]", x),
            Instruction::Unknown(x) => write!(f, "UWN[{:#03x}]", x),
        }
    }
}

impl Instruction {
    fn execute(&self, cpu: &mut Cpu, backend: &Backend) -> Result<(), Error> {
        match self {
            Instruction::Sys(address) => {
                cpu.state.pc = *address as u16;
                Ok(())
            }
            Instruction::Cls => {
                cpu.state.frame_buffer = [false; FRAME_DIMENSIONS.0 * FRAME_DIMENSIONS.1];
                cpu.send_frame();
                Ok(())
            }
            Instruction::Return => {
                cpu.state.sp = cpu.state.sp.saturating_sub(1);
                cpu.state.pc = cpu.state.stack[cpu.state.sp as usize];
                Ok(())
            }
            Instruction::Jump(address) => {
                cpu.state.pc = *address as u16;
                Ok(())
            }
            Instruction::Call(address) => {
                cpu.state.stack[cpu.state.sp as usize] = cpu.state.pc;
                cpu.state.sp = cpu.state.sp.saturating_add(1);
                cpu.state.pc = *address as u16;
                Ok(())
            }
            Instruction::LoadVImmediate(v, value) => {
                cpu.state.v[*v] = *value;
                Ok(())
            }
            Instruction::AddVImmediate(v, value) => {
                cpu.state.v[*v] += *value;
                Ok(())
            }
            Instruction::LoadIImmediate(address) => {
                cpu.state.i = *address as u16;
                Ok(())
            }
            Instruction::Draw(vx, vy, n) => {
                let (start_x, start_y) = (
                    cpu.state.v[*vx] as usize % FRAME_DIMENSIONS.0,
                    cpu.state.v[*vy] as usize % FRAME_DIMENSIONS.1,
                );
                cpu.state.v[0xF] = 0;
                for y in 0..*n {
                    if start_y + y >= FRAME_DIMENSIONS.1 {
                        continue;
                    }
                    let pixeldata = backend
                        .get_bus()
                        .read_u8((cpu.state.i as usize + y) as MemoryAddress)?;
                    for x in 0..8 {
                        if start_x + x >= FRAME_DIMENSIONS.0 {
                            break;
                        }
                        let index = (start_y + y) * FRAME_DIMENSIONS.0 + start_x + x;

                        let current_pixel = cpu.state.frame_buffer[index];
                        let new_pixel = ((pixeldata >> (7 - x)) & 0b1) > 0;

                        cpu.state.frame_buffer[index] = new_pixel != current_pixel;

                        if new_pixel && current_pixel {
                            cpu.state.v[0xF] = 1;
                        }
                    }
                }
                cpu.send_frame();
                Ok(())
            }
            Instruction::Unknown(op) => Err(Error::Emulator(
                axwemulator_core::error::EmulatorErrorKind::UnknownOpcode,
                format!("{:#05x}", op),
            )),
            _ => unimplemented!(),
        }
    }
}
