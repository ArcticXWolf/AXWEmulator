mod audio;
mod cpu;
mod input;
mod timer;

use audio::{AUDIO_SAMPLING_RATE, Audio};
use axwemulator_core::{
    backend::{
        Backend,
        component::{Addressable, Component, MemoryAddress},
        memory::MemoryBlock,
    },
    error::Error,
    frontend::{
        Frontend, audio::build_audio_channel, graphics::build_frame_channel,
        input::build_input_channel,
    },
};
use cpu::{Cpu, FRAME_DIMENSIONS};
use timer::Timer;

const TIMER_BASE: MemoryAddress = 0x100;
const DT_TIMER: MemoryAddress = TIMER_BASE;
const ST_TIMER: MemoryAddress = TIMER_BASE + 1;

const FONT_BASE: MemoryAddress = 0x50;
// From http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.5
#[rustfmt::skip]
const FONT_SET: [u8; 80]  = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub enum Platform {
    Chip8,
    SuperChip,
}

pub struct Chip8Options {
    pub rom_data: Vec<u8>,
    pub platform: Platform,
}

pub fn create_chip8_backend<F: Frontend>(
    frontend: &mut F,
    options: Chip8Options,
) -> Result<Backend, Error> {
    let mut backend = Backend::default();
    let (frame_sender, frame_receiver) =
        build_frame_channel(FRAME_DIMENSIONS.0, FRAME_DIMENSIONS.1);
    let (input_sender, input_receiver) = build_input_channel();
    let (audio_sender, audio_receiver) = build_audio_channel(AUDIO_SAMPLING_RATE, 5000);

    let mut interpreter_memory: MemoryBlock = vec![].into();
    interpreter_memory.resize(0x200);
    interpreter_memory.write(FONT_BASE, &FONT_SET)?;
    backend.add_addressable_component("mem_interpreter", 0x0, Component::new(interpreter_memory));

    let mut ram: MemoryBlock = options.rom_data.into();
    ram.resize(0xFFF - 0x200);
    backend.add_addressable_component("mem_ram", 0x200, Component::new(ram));

    let timer = Timer::new();
    backend.add_component("timer", Component::new(timer));

    let cpu = Cpu::new(options.platform, frame_sender, input_receiver);
    backend.add_component("cpu", Component::new(cpu));
    frontend.register_input_sender(input_sender)?;
    frontend.register_graphics_receiver(frame_receiver)?;

    let audio = Audio::new(audio_sender);
    backend.add_component("audio", Component::new(audio));
    frontend.register_audio_receiver(audio_receiver)?;

    Ok(backend)
}
