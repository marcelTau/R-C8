use crate::cpu::Cpu;
use crate::graphics::Graphics;
use std::error::Error;
use std::fs;
use std::io::Read;

pub const COL: usize = 64;
pub const ROW: usize = 32;
pub const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    cpu: Cpu,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 { cpu: Cpu::new() }
    }

    pub fn load_program(&mut self, program_name: &str) -> Result<usize, Box<dyn Error>> {
        let mut file = fs::File::open(program_name)?;
        let read_bytes = file.read(&mut self.cpu.memory[512..])?;
        println!("Read {} bytes from file {}", read_bytes, program_name);
        self.cpu.load_font();
        Ok(read_bytes)
    }

    pub fn gameloop(&mut self) {
        let mut graphics = Graphics::new();
        while graphics.app.next_frame() {
            let opcode = self.cpu.fetch_opcode();
            self.cpu
                .decode_and_execute_graphic(opcode, Some(&mut graphics));
            self.cpu.update_timers();
            self.cpu.update_timers();
            if self.cpu.should_redraw {
                graphics.draw(&self.cpu.graphics);
                self.cpu.should_redraw = false;
            }
        }
    }
}
