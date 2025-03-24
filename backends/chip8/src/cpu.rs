use std::fmt::Display;

use axwemulator_core::{
    backend::{
        Backend,
        component::{Addressable, Inspectable, MemoryAddress, Steppable, Transmutable},
    },
    error::Error,
    frontend::{
        graphics::{Frame, FrameSender},
        input::{ButtonState, InputEvent, InputReceiver},
    },
};
use femtos::Duration;

use crate::{
    DT_TIMER, FONT_BASE, Platform, ST_TIMER,
    input::{InputButton, KeypadState},
};

pub const CLOCK_SPEED_NS: u64 = 1_000_000_000 / 700;
pub const VBLANK_CLOCK_SPEED_NS: u64 = 1_000_000_000 / 60;
pub const FRAME_DIMENSIONS: (usize, usize) = (64, 32);

#[derive(Default)]
pub struct CpuQuirks {
    quirks_shift_takes_x_instead_of_y: bool,
    quirks_loadstore_leaves_i_unmodified: bool,
    quirks_loadstore_modifies_i_one_less: bool,
    quirks_jump_uses_x: bool,
    quirks_draw_not_waiting_for_vblank: bool,
    quirks_logic_leaves_flag_unmodified: bool,
}

impl From<Platform> for CpuQuirks {
    fn from(value: Platform) -> Self {
        match value {
            Platform::Chip8 => Self {
                quirks_shift_takes_x_instead_of_y: false,
                quirks_loadstore_leaves_i_unmodified: false,
                quirks_loadstore_modifies_i_one_less: false,
                quirks_jump_uses_x: false,
                quirks_draw_not_waiting_for_vblank: false,
                quirks_logic_leaves_flag_unmodified: false,
            },
            Platform::SuperChip => Self {
                quirks_shift_takes_x_instead_of_y: true,
                quirks_loadstore_leaves_i_unmodified: true,
                quirks_loadstore_modifies_i_one_less: false,
                quirks_jump_uses_x: true,
                quirks_draw_not_waiting_for_vblank: true,
                quirks_logic_leaves_flag_unmodified: true,
            },
        }
    }
}

pub struct CpuState {
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    paused: bool,
    waiting_for_key: Option<usize>,
    waiting_for_vblank: bool,
    frame_buffer: [bool; FRAME_DIMENSIONS.0 * FRAME_DIMENSIONS.1],
    keypad_state: KeypadState,
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            v: Default::default(),
            i: Default::default(),
            pc: Default::default(),
            sp: Default::default(),
            stack: Default::default(),
            paused: Default::default(),
            waiting_for_key: Default::default(),
            waiting_for_vblank: Default::default(),
            frame_buffer: [false; FRAME_DIMENSIONS.0 * FRAME_DIMENSIONS.1],
            keypad_state: KeypadState::new(),
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

impl Display for CpuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v_str = String::new();
        let mut stack_str = String::new();
        for (i, v) in self.v.iter().enumerate() {
            v_str = format!("{}{:01x}:{:#04x} ", v_str, i, *v);
        }
        for s in self.stack.iter() {
            stack_str = format!("{}{:#06x} ", stack_str, *s);
        }
        write!(
            f,
            "I:{:#06x} PC:{:#06x} V:[{}] SP:{:02}",
            self.i, self.pc, v_str, self.sp
        )
    }
}

#[derive(Default)]
pub struct Cpu {
    state: CpuState,
    quirks: CpuQuirks,
    frame_sender: Option<FrameSender>,
    input_receiver: Option<InputReceiver>,
}

impl Cpu {
    pub fn new(
        platform: Platform,
        frame_sender: FrameSender,
        input_receiver: InputReceiver,
    ) -> Self {
        Self {
            state: CpuState::new(),
            quirks: platform.into(),
            frame_sender: Some(frame_sender),
            input_receiver: Some(input_receiver),
        }
    }

    fn handle_input(&mut self) {
        while let Some(ie) = self.input_receiver.as_ref().unwrap().pop() {
            self.state.keypad_state.parse_input_event(ie);

            if let Some(x) = self.state.waiting_for_key {
                match ie {
                    InputEvent::Keyboard(keyboard_event_key, ButtonState::Released) => {
                        if let Ok(button) = InputButton::try_from(keyboard_event_key) {
                            self.state.v[x] = button.into();
                            self.state.waiting_for_key = None;
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn send_frame(&self, backend: &Backend) {
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

        self.frame_sender
            .as_ref()
            .unwrap()
            .add(backend.get_current_clock(), frame);
    }
}

impl Steppable for Cpu {
    fn step(&mut self, backend: &Backend) -> Result<Duration, Error> {
        self.handle_input();

        if !self.state.paused && self.state.waiting_for_key.is_none() {
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

        if !self.quirks.quirks_draw_not_waiting_for_vblank && self.state.waiting_for_vblank {
            let last_vblank_idx = backend.get_current_clock().as_duration()
                / Duration::from_nanos(VBLANK_CLOCK_SPEED_NS);
            let next_vblank = Duration::from_nanos((last_vblank_idx + 1) * VBLANK_CLOCK_SPEED_NS);
            let next_cpu_clock = next_vblank
                .checked_sub(backend.get_current_clock().as_duration())
                .unwrap();
            self.state.waiting_for_vblank = false;
            Ok(next_cpu_clock)
        } else {
            Ok(Duration::from_nanos(CLOCK_SPEED_NS))
        }
    }
}

impl Inspectable for Cpu {
    fn inspect(&self) -> Vec<String> {
        let mut result = vec![];
        result.push(format!("{:>6}: {}", "PC", self.state.pc));
        result.push(format!("{:>6}: {}", "SP", self.state.sp));
        result.push(format!("{:>6}: {}", "I", self.state.i));
        for (i, r) in self.state.v.iter().enumerate() {
            result.push(format!("{:>6}: {}", format!("v[{}]", i), r));
        }
        for (i, r) in self.state.stack.iter().enumerate() {
            result.push(format!("{:>6}: {}", format!("s[{}]", i), r));
        }
        result
    }
}

impl Transmutable for Cpu {
    fn as_steppable(&mut self) -> Option<&mut dyn Steppable> {
        Some(self)
    }

    fn as_inspectable(&mut self) -> Option<&mut dyn Inspectable> {
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
                cpu.send_frame(backend);
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
            Instruction::SkipIfVImmediate(x, y) => {
                if cpu.state.v[*x] == *y {
                    cpu.state.pc += 2;
                }
                Ok(())
            }
            Instruction::SkipIfNotVImmediate(x, y) => {
                if cpu.state.v[*x] != *y {
                    cpu.state.pc += 2;
                }
                Ok(())
            }
            Instruction::SkipIfCmp(x, y) => {
                if cpu.state.v[*x] == cpu.state.v[*y] {
                    cpu.state.pc += 2;
                }
                Ok(())
            }
            Instruction::LoadVImmediate(v, value) => {
                cpu.state.v[*v] = *value;
                Ok(())
            }
            Instruction::AddVImmediate(v, value) => {
                cpu.state.v[*v] = cpu.state.v[*v].wrapping_add(*value);
                Ok(())
            }
            Instruction::LoadV(x, y) => {
                cpu.state.v[*x] = cpu.state.v[*y];
                Ok(())
            }
            Instruction::Or(x, y) => {
                cpu.state.v[*x] |= cpu.state.v[*y];
                if !cpu.quirks.quirks_logic_leaves_flag_unmodified {
                    cpu.state.v[0xF] = 0;
                }
                Ok(())
            }
            Instruction::And(x, y) => {
                cpu.state.v[*x] &= cpu.state.v[*y];
                if !cpu.quirks.quirks_logic_leaves_flag_unmodified {
                    cpu.state.v[0xF] = 0;
                }
                Ok(())
            }
            Instruction::Xor(x, y) => {
                cpu.state.v[*x] ^= cpu.state.v[*y];
                if !cpu.quirks.quirks_logic_leaves_flag_unmodified {
                    cpu.state.v[0xF] = 0;
                }
                Ok(())
            }
            Instruction::Add(x, y) => {
                let (result, overflow) = cpu.state.v[*x].overflowing_add(cpu.state.v[*y]);
                cpu.state.v[*x] = result;
                cpu.state.v[0xF] = if overflow { 1 } else { 0 };
                Ok(())
            }
            Instruction::Sub(x, y) => {
                let (result, overflow) = cpu.state.v[*x].overflowing_sub(cpu.state.v[*y]);
                cpu.state.v[*x] = result;
                cpu.state.v[0xF] = if overflow { 0 } else { 1 };
                Ok(())
            }
            Instruction::ShiftRight(x, y) => {
                if !cpu.quirks.quirks_shift_takes_x_instead_of_y {
                    cpu.state.v[*x] = cpu.state.v[*y];
                }
                let flag = cpu.state.v[*x] & 0b1;
                cpu.state.v[*x] >>= 1;
                cpu.state.v[0xF] = flag;
                Ok(())
            }
            Instruction::SubN(x, y) => {
                let (result, overflow) = cpu.state.v[*y].overflowing_sub(cpu.state.v[*x]);
                cpu.state.v[*x] = result;
                cpu.state.v[0xF] = if overflow { 0 } else { 1 };
                Ok(())
            }
            Instruction::ShiftLeft(x, y) => {
                if !cpu.quirks.quirks_shift_takes_x_instead_of_y {
                    cpu.state.v[*x] = cpu.state.v[*y];
                }
                let flag = cpu.state.v[*x] & 0b10000000;
                cpu.state.v[*x] <<= 1;
                cpu.state.v[0xF] = flag >> 7;
                Ok(())
            }
            Instruction::SkipIfNotCmp(x, y) => {
                if cpu.state.v[*x] != cpu.state.v[*y] {
                    cpu.state.pc += 2;
                }
                Ok(())
            }
            Instruction::LoadIImmediate(address) => {
                cpu.state.i = *address as u16;
                Ok(())
            }
            Instruction::JumpV0(x) => {
                if cpu.quirks.quirks_jump_uses_x {
                    let register = *x & 0xF00;
                    cpu.state.pc = (cpu.state.v[register] as u16).wrapping_add(*x as u16);
                } else {
                    cpu.state.pc = (cpu.state.v[0x0] as u16).wrapping_add(*x as u16);
                }
                Ok(())
            }
            Instruction::Random(x, y) => {
                let random: u8 = rand::random();
                cpu.state.v[*x] = random & *y;
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
                cpu.send_frame(backend);
                if !cpu.quirks.quirks_draw_not_waiting_for_vblank {
                    cpu.state.waiting_for_vblank = true;
                }
                Ok(())
            }
            Instruction::SkipIfKey(x) => {
                if cpu
                    .state
                    .keypad_state
                    .get_state_for_button(cpu.state.v[*x].try_into().unwrap())
                    == ButtonState::Pressed
                {
                    cpu.state.pc += 2;
                };
                Ok(())
            }
            Instruction::SkipIfNotKey(x) => {
                if cpu
                    .state
                    .keypad_state
                    .get_state_for_button(cpu.state.v[*x].try_into().unwrap())
                    == ButtonState::Released
                {
                    cpu.state.pc += 2;
                };
                Ok(())
            }
            Instruction::LoadVDT(x) => {
                cpu.state.v[*x] = backend.get_bus().read_u8(DT_TIMER)?;
                Ok(())
            }
            Instruction::WaitAndLoadKeypress(x) => {
                // just set wait for key and return. the rest is handled in the step function
                cpu.state.waiting_for_key = Some(*x);
                Ok(())
            }
            Instruction::LoadDTV(x) => {
                backend.get_bus().write_u8(DT_TIMER, cpu.state.v[*x])?;
                Ok(())
            }
            Instruction::LoadSTV(x) => {
                backend.get_bus().write_u8(ST_TIMER, cpu.state.v[*x])?;
                Ok(())
            }
            Instruction::AddIV(x) => {
                cpu.state.i = cpu.state.i.wrapping_add(cpu.state.v[*x] as u16);
                Ok(())
            }
            Instruction::LoadFontV(x) => {
                cpu.state.i = FONT_BASE as u16 + (*x as u16) * 5;
                Ok(())
            }
            Instruction::StoreBCDV(x) => {
                let hundreds = (cpu.state.v[*x] / 100) % 10;
                let tens = (cpu.state.v[*x] / 10) % 10;
                let ones = cpu.state.v[*x] % 10;
                backend.get_bus().write_u8(cpu.state.i as usize, hundreds)?;
                backend
                    .get_bus()
                    .write_u8((cpu.state.i + 1) as usize, tens)?;
                backend
                    .get_bus()
                    .write_u8((cpu.state.i + 2) as usize, ones)?;
                Ok(())
            }
            Instruction::StoreAllV(x) => {
                for register in 0..=*x {
                    backend
                        .get_bus()
                        .write_u8(cpu.state.i as usize + register, cpu.state.v[register])?;
                }
                if !cpu.quirks.quirks_loadstore_leaves_i_unmodified {
                    cpu.state.i += *x as u16;
                    if !cpu.quirks.quirks_loadstore_modifies_i_one_less {
                        cpu.state.i += 1;
                    }
                }
                Ok(())
            }
            Instruction::LoadAllV(x) => {
                for register in 0..=*x {
                    cpu.state.v[register] =
                        backend.get_bus().read_u8(cpu.state.i as usize + register)?;
                }
                if !cpu.quirks.quirks_loadstore_leaves_i_unmodified {
                    cpu.state.i += *x as u16;
                    if !cpu.quirks.quirks_loadstore_modifies_i_one_less {
                        cpu.state.i += 1;
                    }
                }
                Ok(())
            }
            Instruction::Unknown(op) => Err(Error::Emulator(
                axwemulator_core::error::EmulatorErrorKind::UnknownOpcode,
                format!("{:#05x}", op),
            )),
        }
    }
}
