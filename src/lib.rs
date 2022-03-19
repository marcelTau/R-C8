#![allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::thread;
use std::time::Duration;

const ROW: usize = 64;
const COL: usize = 32;
const FONTSET: [u8; 80] = [
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
    stack: [u16; 16],
    sp: u16,
    keypad: [u8; 16],
    memory: [u8; 4096],
    v: [u8; 16],
    graphics: [u8; ROW * COL],
    i: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    opcode_function: HashMap<u16, F>,
}

type F = fn(&mut Chip8, u16);

impl Chip8 {
    pub fn new() -> Self {
        let m = HashMap::<u16, F>::new();
        Chip8 {
            stack: [0; 16],
            sp: 0,
            keypad: [0; 16],
            memory: [0; 4096],
            v: [0; 16],
            graphics: [0; ROW * COL],
            i: 0,
            pc: 0x200, //
            delay_timer: 0,
            sound_timer: 0,
            opcode_function: m,
        }
    }

    fn clear_screen(&mut self) {
        self.graphics.fill(0);
    }

    fn return_from_subroutine(&mut self) {
        if self.sp == 0 && self.stack[0] == 0 {
            panic!("Stack is empty");
        }

        if self.sp > 0 {
            self.sp -= 1;
        }

        self.pc = self.stack[self.sp as usize];
        self.stack[self.sp as usize] = 0;
    }

    fn f_0x0000(&mut self, opcode: u16) {
        match opcode & 0x000F {
            // 0x00E0: Clears the screen
            0x0000 => self.clear_screen(),

            // 0x00EE: Returns from subroutine
            0x000E => self.return_from_subroutine(),
            _ => eprintln!("Opcode '{:#X}' not found", opcode),
        }
        println!("working: {}", opcode);
    }

    // 0x1NNN: Jumps to address NNN
    fn f_0x1000(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = address;
    }

    fn setup_map(&mut self) {
        self.opcode_function.insert(0x0000, Chip8::f_0x0000);
        self.opcode_function.insert(0x1000, Chip8::f_0x1000);
    }

    pub fn load_game(&mut self, filename: &str) -> Result<usize, Box<dyn Error>> {
        let mut file = fs::File::open(filename)?;
        let read_bytes = file.read(&mut self.memory[512..])?;
        println!("Read {} bytes from file {}", read_bytes, filename);
        Ok(read_bytes)
    }

    pub fn load_font(&mut self) {
        for (i, &value) in FONTSET.iter().enumerate() {
            self.memory[i] = value;
        }
    }

    pub fn handle_opcode(&mut self, opcode: u16) {
        match self.opcode_function.get(&(opcode & 0xF000)) {
            Some(func) => func(self, opcode),
            None => {
                eprintln!("Opcode '{:#X}' not found", opcode);
            }
        }
    }

    fn emulate_cycle(&mut self) {
        let mut first: u16 = self.memory[self.pc as usize].into();
        first = first << 8;
        let second: u16 = (self.memory[self.pc as usize + 1]).into();
        let opcode: u16 = first | second;

        self.handle_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            println!("some sound ...");
        }
    }

    pub fn run(&mut self) {
        loop {
            self.emulate_cycle();
            thread::sleep(Duration::from_secs(2));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_chip() -> Chip8 {
        let mut chip = Chip8::new();
        chip.setup_map();
        chip.load_font();
        chip.load_game("pong.rom").unwrap();
        chip
    }

    #[test]
    fn load_game() {
        let mut chip = Chip8::new();
        let ret = chip.load_game("pong.rom").unwrap();
        assert_eq!(ret, 246);
    }

    #[test]
    fn load_font() {
        let mut chip = Chip8::new();
        chip.load_font();
        assert_eq!(chip.memory[..80], FONTSET);
    }

    #[test]
    fn clear_screen() {
        let mut chip = create_chip();
        chip.graphics.fill(1);
        chip.handle_opcode(0x00E0);
        assert_eq!(chip.graphics, [0; ROW * COL]);
    }

    #[test]
    #[should_panic(expected = "Stack is empty")]
    fn return_from_subroutine_empty_stack() {
        let mut chip = create_chip();
        chip.return_from_subroutine();
    }

    #[test]
    fn return_from_subroutine() {
        let mut chip = create_chip();

        // jump to 128
        chip.handle_opcode(0x1080);
        assert_eq!(chip.pc, 128);

        // jump to 131
        chip.handle_opcode(0x1083);
        assert_eq!(chip.pc, 131);

        // jump to 132
        chip.handle_opcode(0x1084);
        assert_eq!(chip.pc, 132);

        // jump back to 128
        chip.handle_opcode(0x00EE);
        assert_eq!(chip.pc, 131);
    }

    #[test]
    #[allow(non_snake_case)]
    fn jump_to_address_0x1NNN() {
        let mut chip = create_chip();
        chip.handle_opcode(0x1080);
        assert_eq!(chip.pc, 128);
    }
}
