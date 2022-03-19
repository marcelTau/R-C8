#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::io::Read;

const ROW: usize = 64;
const COL: usize = 32;

struct Chip8 {
    stack: [u16; 16],
    stack_pointer: u16,
    keypad: [u8; 16],
    memory: [u8; 4096],
    registers: [u8; 16],
    graphics: [u8; ROW * COL],
    index_register: u16,
    program_counter: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            stack: [0; 16],
            stack_pointer: 0,
            keypad: [0; 16],
            memory: [0; 4096],
            registers: [0; 16],
            graphics: [0; ROW * COL],
            index_register: 0,
            program_counter: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn load_game(&mut self, filename: &str) -> Result<usize, Box<dyn Error>> {
        let mut file = fs::File::open(filename)?;
        let read_bytes = file.read(&mut self.memory[512..])?;
        println!("Read {} bytes from file {}", read_bytes, filename);
        Ok(read_bytes)
    }

    fn run(&mut self) {
        loop {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_game() {
        let mut chip = Chip8::new();
        let ret = chip.load_game("pong.rom").unwrap();
        assert_eq!(ret, 246);
    }
}

fn main() {
    let mut chip = Chip8::new();

    chip.load_game("pong.rom").unwrap_or_else(|err| {
        eprintln!("Error occured during loading the program: {}", err);
        std::process::exit(1);
    });

    println!("Hello, world!");
}
