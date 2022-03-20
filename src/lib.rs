#![allow(dead_code)]
#![allow(non_snake_case)]

use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::thread;
use std::time::Duration;

const COL: usize = 64;
const ROW: usize = 32;
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
    should_redraw: bool,
}

type F = fn(&mut Chip8, u16);

// initialization
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
            should_redraw: false,
        }
    }
    pub fn setup_map(&mut self) {
        self.opcode_function.insert(0x0000, Chip8::f_0x0000);
        self.opcode_function.insert(0x1000, Chip8::f_0x1000);
        self.opcode_function.insert(0x2000, Chip8::f_0x2000);
        self.opcode_function.insert(0x3000, Chip8::f_0x3000);
        self.opcode_function.insert(0x4000, Chip8::f_0x4000);
        self.opcode_function.insert(0x5000, Chip8::f_0x5000);
        self.opcode_function.insert(0x6000, Chip8::f_0x6000);
        self.opcode_function.insert(0x7000, Chip8::f_0x7000);
        self.opcode_function.insert(0x8000, Chip8::f_0x8000);
        self.opcode_function.insert(0x9000, Chip8::f_0x9000);
        self.opcode_function.insert(0xA000, Chip8::f_0xA000);
        self.opcode_function.insert(0xB000, Chip8::f_0xB000);
        self.opcode_function.insert(0xC000, Chip8::f_0xC000);
        self.opcode_function.insert(0xD000, Chip8::f_0xD000);
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
}

impl Chip8 {
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
    }

    // 0x1NNN: Jumps to address NNN
    fn f_0x1000(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;

        println!("Jumping to address {:#X}", address);

        //self.stack[self.sp as usize] = self.pc; //todo
        //self.sp += 1;
        self.pc = address;
    }

    // 0x2NNN Calls subroutine at NNN and saves the current address on the stack
    fn f_0x2000(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;

        self.pc = address;
    }

    // 0x3XNN skips the next instruction if v[X] == NN
    fn f_0x3000(&mut self, opcode: u16) {
        let index = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;

        if self.v[index as usize] == value as u8 {
            self.pc += 2;
        }
    }

    // 0x4XNN skips the next instruction if v[X] != NN
    fn f_0x4000(&mut self, opcode: u16) {
        let index = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;

        if self.v[index as usize] != value as u8 {
            self.pc += 2;
        }
    }

    // 0x5XY0 skips the next instruction if v[X] == v[Y]
    fn f_0x5000(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        if self.v[X as usize] == self.v[Y as usize] {
            self.pc += 2;
        }
    }

    // 0x6XNN set value v[X] = NN
    fn f_0x6000(&mut self, opcode: u16) {
        let index: u16 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
        let value: u8 = (opcode & 0x00FF).try_into().unwrap();
        self.v[index as usize] = value;
    }

    // 0x7XNN add NN to v[X]
    fn f_0x7000(&mut self, opcode: u16) {
        let index: u16 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
        let value: u8 = (opcode & 0x00FF).try_into().unwrap();
        self.v[index as usize] += value;
    }

    // 0x8000
    fn f_0x8000(&mut self, opcode: u16) {
        match opcode & 0x000F {
            0 => self.f_0x8XY0(opcode),
            1 => self.f_0x8XY1(opcode),
            2 => self.f_0x8XY2(opcode),
            3 => self.f_0x8XY3(opcode),
            4 => self.f_0x8XY4(opcode),
            5 => self.f_0x8XY5(opcode),
            6 => self.f_0x8XY6(opcode),
            7 => self.f_0x8XY7(opcode),
            0xE => self.f_0x8XYE(opcode),
            _ => unreachable!(),
        }
    }

    // 0x8XY0 set v[X] = v[Y]
    fn f_0x8XY0(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        self.v[X as usize] = self.v[Y as usize];
    }

    // 0x8XY1 sets v[X] = v[X] | v[Y]
    fn f_0x8XY1(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        self.v[X as usize] |= self.v[Y as usize];
    }

    // 0x8XY2 sets v[X] = v[X] & v[Y]
    fn f_0x8XY2(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        self.v[X as usize] &= self.v[Y as usize];
    }

    // 0x8XY3 sets v[X] = v[X] ^ v[Y]
    fn f_0x8XY3(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        self.v[X as usize] ^= self.v[Y as usize];
    }

    // 0x8XY4 sets v[X] = v[X] + v[Y] and set v[0xF] to 1 if there is a carry
    fn f_0x8XY4(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        let sum: u16 = self.v[X as usize] as u16 + self.v[Y as usize] as u16;

        if sum > u8::MAX.into() {
            self.v[X as usize] = (sum >> 8) as u8;
            self.v[0xF] = 0x1;
            return;
        }

        self.v[0xF] = 0x0;
        self.v[X as usize] = sum as u8;
    }

    // 0x8XY5 sets v[X] = v[X] - v[Y] and set v[0xF] t0 0x0 if there is a borrow and to 0x1 if not
    fn f_0x8XY5(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        if self.v[X as usize] >= self.v[Y as usize] {
            self.v[0xF] = 0x1;
        } else {
            self.v[0xF] = 0x0;
        }
        self.v[X as usize] = self.v[X as usize].wrapping_add(self.v[Y as usize].wrapping_neg());
    }

    // 0x8XY6 stores the least significant bit of v[X] in v[0xF] then shifts v[X] to the right by 1
    fn f_0x8XY6(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;
        // store least significant bit of v[X] in v[0xF]
        self.v[0xF] = self.v[X as usize] & 0x1;
        self.v[X as usize] = self.v[Y as usize] >> 1;
    }

    // 0x8XY7 sets v[X] = v[Y] - v[X], and set v[0xF] to 0 if there is a borrow if not then 1
    fn f_0x8XY7(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        if self.v[X as usize] > self.v[Y as usize] {
            self.v[0xF] = 0x0;
        } else {
            self.v[0xF] = 0x1;
        }
        self.v[X as usize] = self.v[Y as usize].wrapping_add(self.v[X as usize].wrapping_neg());
    }

    // 0x8XYE stores the MOST significant bit of v[Y] in v[0xF]
    // then sets v[X] to v[Y] <<= 1
    fn f_0x8XYE(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        self.v[0xF] = (self.v[Y as usize] & 0b10000000) >> 7;
        self.v[X as usize] = self.v[Y as usize].wrapping_shl(1);
    }

    // 0x9XY0 skips the next instruction if v[X] != v[Y]
    fn f_0x9000(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let Y = (opcode & 0x00F0) >> 4;

        if self.v[X as usize] != self.v[Y as usize] {
            self.pc += 2;
        }
    }

    //0xANNN sets self.i = NNN
    fn f_0xA000(&mut self, opcode: u16) {
        self.i = opcode & 0x0FFF;
    }

    // 0xBNNN jumps to address NNN + v[0]
    fn f_0xB000(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) + self.v[0] as u16;
    }

    // 0xCXNN set v[X] to rand(1..255) & NN
    fn f_0xC000(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        let NN = opcode & 0x00FF;
        let r: u8 = rand::thread_rng().gen_range(1..=255);
        self.v[X as usize] = r & NN as u8;
    }

    // 0xDXYN: draw sprite at coordinate X,Y with height of N
    fn f_0xD000(&mut self, opcode: u16) {
        let x: u16 = self.v[((opcode & 0x0F00) >> 8) as usize].into();
        let y: u16 = self.v[((opcode & 0x00F0) >> 4) as usize].into();
        let height: u16 = opcode & 0x000F;

        self.v[0xF] = 0;

        for yline in 0..height {
            let pixel = self.memory[(self.i + yline) as usize];
            for xline in 0..8 {
                if pixel & (0x80 >> xline) != 0 {
                    let index = x + xline + ((y + yline) * (COL as u16));
                    if self.graphics[index as usize] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.graphics[index as usize] ^= 1;
                }
            }
        }
        self.should_redraw = true;
    }

    fn f_0xE000(&mut self, opcode: u16) {
        match opcode & 0x00FF {
            0x9E => todo!(),
            0xA1 => todo!(),
            _ => unreachable!(),
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

    fn fetch_opcode(&mut self) -> u16 {
        let left: u16 = self.memory[self.pc as usize].into();
        let right: u16 = (self.memory[self.pc as usize + 1]).into();
        let opcode: u16 = (left << 8) | right;
        self.pc += 2;

        opcode
    }

    fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();

        self.handle_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            println!("some sound ...");
        }
    }

    fn redraw(&mut self, app: &mut simple::Window) {
        if self.should_redraw == false {
            return;
        }

        for (i, &value) in self.graphics.iter().enumerate() {
            if value == 0 {
                continue;
            }
            let x = i % COL;
            let y = i / ROW;

            let r = simple::Point::new(x as i32, y as i32);
            app.draw_point(r);
            app.set_color(255, 255, 255, 255);
        }

        self.should_redraw = false;
    }

    pub fn run(&mut self) {
        let mut app = simple::Window::new("Chip8", (COL * 10) as u16, (ROW * 10) as u16);

        while app.next_frame() {
            self.emulate_cycle();
            self.redraw(&mut app);
            thread::sleep(Duration::from_millis(1000 / 60));
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
    fn clear_screen2() {
        let mut chip = create_chip();
        chip.graphics.fill(1);
        assert_eq!(chip.graphics, [1; ROW * COL]);
        chip.handle_opcode(0xE0);
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
        // call subroutine at address 0x80 and put current address of 0x200 on the stack
        chip.handle_opcode(0x2080);
        assert_eq!(chip.pc, 0x80);

        println!("{:?}", chip.stack);
        // call subroutine at address 0x83 and put current address of 0x80 on the stack
        chip.handle_opcode(0x2083);
        assert_eq!(chip.pc, 0x83);
        println!("{:?}", chip.stack);

        // return from subroutine at address 0x83 and go back to 0x80
        chip.handle_opcode(0x00EE);
        assert_eq!(chip.pc, 0x80);
        println!("{:?}", chip.stack);
    }

    #[test]
    fn jump_to_address_0x1NNN() {
        let mut chip = create_chip();
        chip.handle_opcode(0x1080);
        assert_eq!(chip.pc, 128);
    }

    #[test]
    fn call_subroutine_0x2NNN() {
        let mut chip = create_chip();

        // call subroutine at address 128, and put current 512 on the stack
        chip.handle_opcode(0x2080);
        assert_eq!(chip.pc, 128);

        // call subroutine at address 5, and put current address 128 on the stack
        chip.handle_opcode(0x2005);
        assert_eq!(chip.pc, 5);

        assert_eq!(chip.stack[chip.sp as usize - 1], 128);
    }

    #[test]
    fn skip_if_equal_0x3XNN() {
        let mut chip = create_chip();
        assert_eq!(chip.pc, 0x200);
        chip.v[2] = 4;

        chip.handle_opcode(0x3205);
        assert_eq!(chip.pc, 0x200);
        chip.handle_opcode(0x3204);

        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn skip_if_not_equal_0x4XNN() {
        let mut chip = create_chip();
        assert_eq!(chip.pc, 0x200);
        chip.v[2] = 4;
        chip.handle_opcode(0x4204);
        assert_eq!(chip.pc, 0x200);

        chip.handle_opcode(0x4205);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn skip_xy_0x5XY0() {
        let mut chip = create_chip();
        chip.v[1] = 10;
        chip.v[4] = 11;

        chip.handle_opcode(0x5140);
        assert_eq!(chip.pc, 0x200);

        chip.v[4] = 10;
        chip.handle_opcode(0x5140);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn set_value_0x6XNN() {
        let mut chip = create_chip();

        // set v[0] = 128
        chip.handle_opcode(0x6080);
        assert_eq!(chip.v[0], 128);

        // set v[10] = 128
        chip.handle_opcode(0x6a80);
        assert_eq!(chip.v[10], 128);

        println!("{:?}", chip.v);
    }

    #[test]
    fn add_to_value_0x7XNN() {
        let mut chip = create_chip();

        // set v[10] = 8
        chip.handle_opcode(0x6a08);
        assert_eq!(chip.v[10], 8);

        chip.handle_opcode(0x7a08);
        assert_eq!(chip.v[10], 16);
    }

    #[test]
    fn assign_value_0x8XY0() {
        let mut chip = create_chip();
        chip.v[3] = 4;
        chip.handle_opcode(0x8430);
        assert_eq!(chip.v[4], 4);
    }

    #[test]
    fn or_value_0x8XY1() {
        let mut chip = create_chip();
        chip.v[8] = 10;
        chip.v[11] = 172;

        chip.handle_opcode(0x88b1);

        assert_eq!(chip.v[8], 10 | 172);
    }

    #[test]
    fn and_value_0x8XY2() {
        let mut chip = create_chip();
        chip.v[8] = 10;
        chip.v[11] = 172;

        chip.handle_opcode(0x88b2);

        assert_eq!(chip.v[8], 10 & 172);
    }

    #[test]
    fn xor_value_0x8XY3() {
        let mut chip = create_chip();
        chip.v[8] = 10;
        chip.v[11] = 172;

        chip.handle_opcode(0x88b3);

        assert_eq!(chip.v[8], 10 ^ 172);
    }

    #[test]
    fn adding_registers_simple_0x8XY4() {
        let mut chip = create_chip();

        chip.v[2] = 5;
        chip.v[3] = 10;

        chip.handle_opcode(0x8234);

        assert_eq!(chip.v[2], 15);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn adding_registers_with_carry_0x8XY4() {
        let mut chip = create_chip();

        chip.v[2] = 255;
        chip.v[3] = 1;

        chip.handle_opcode(0x8234);

        assert_eq!(chip.v[2], 0x1);
        assert_eq!(chip.v[0xF], 0x1);
    }

    #[test]
    fn subtracting_registers_simple_0x8XY5() {
        let mut chip = create_chip();

        chip.v[2] = 10;
        chip.v[3] = 5;

        chip.handle_opcode(0x8235);

        assert_eq!(chip.v[2], 5);
        assert_eq!(chip.v[0xF], 0x1);
    }

    #[test]
    fn subtracting_registers_with_borrow_0x8XY5() {
        let mut chip = create_chip();

        chip.v[2] = 5;
        chip.v[3] = 10;

        chip.handle_opcode(0x8235);

        assert_eq!(chip.v[2], 251);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn right_shifting_0x8XY6() {
        let mut chip = create_chip();

        chip.v[2] = 3;
        chip.v[3] = 5;
        chip.handle_opcode(0x8326);
        assert_eq!(chip.v[3], 0x1);
        assert_eq!(chip.v[0xF], 0x1);

        chip.v[2] = 3;
        chip.v[3] = 4;
        chip.handle_opcode(0x8326);
        assert_eq!(chip.v[3], 0x1);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn subtracting_y_x_simple_0x8XY7() {
        let mut chip = create_chip();

        chip.v[2] = 3;
        chip.v[3] = 1;

        chip.handle_opcode(0x8327);

        assert_eq!(chip.v[3], 2);
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn subtracting_y_x_with_borrow_0x8XY7() {
        let mut chip = create_chip();

        chip.v[2] = 3;
        chip.v[3] = 10;

        chip.handle_opcode(0x8327);

        assert_eq!(chip.v[3], 249);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn left_shifting_0x8XYE() {
        let mut chip = create_chip();

        chip.v[3] = 4; //X
        chip.v[2] = 128; //Y

        chip.handle_opcode(0x832E);

        assert_eq!(chip.v[0xF], 0x1);
        assert_eq!(chip.v[3], 0x0);
    }

    #[test]
    fn skip_if_xy_not_equal_0x9000() {
        let mut chip = create_chip();

        chip.v[2] = 5;
        chip.v[3] = 5;
        chip.handle_opcode(0x9230);

        assert_eq!(chip.pc, 0x200);

        chip.v[3] = 6;
        chip.handle_opcode(0x9230);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn set_index_register_value_0xA000() {
        let mut chip = create_chip();

        chip.handle_opcode(0xA001);
        assert_eq!(chip.i, 1);

        chip.handle_opcode(0xA123);
        assert_eq!(chip.i, 291);
    }

    #[test]
    fn jump_to_nnn_plus_v0_0xB000() {
        let mut chip = create_chip();
        assert_eq!(chip.pc, 0x200);
        chip.v[0] = 0x80;
        chip.handle_opcode(0xB080);
        assert_eq!(chip.pc, 0x80 + 0x80);
    }
}
