#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::chip8::{COL, FONTSET, ROW};
use crate::graphics::Graphics;
use rand::Rng;
use std::collections::HashMap;

const KEYMAP: [simple::Key; 16] = [
    simple::Key::A,
    simple::Key::S,
    simple::Key::D,
    simple::Key::F,
    simple::Key::Up,
    simple::Key::Right,
    simple::Key::Down,
    simple::Key::Left,
    simple::Key::Num1,
    simple::Key::Num2,
    simple::Key::Num3,
    simple::Key::Num4,
    simple::Key::Num5,
    simple::Key::Num6,
    simple::Key::Num7,
    simple::Key::Num8,
];

pub struct Cpu {
    pub graphics: [u8; ROW * COL],
    pub memory: [u8; 4096],
    pub should_redraw: bool,
    stack: [u16; 16],
    sp: u16,
    v: [u8; 16],
    i: u16,
    pc: u16,
    keypad: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    opcode_function: HashMap<u16, OpcodeFunction>,
}
type OpcodeFunction = fn(&mut Cpu, u16);

impl Cpu {
    pub fn new() -> Cpu {
        let mut map: HashMap<u16, OpcodeFunction> = HashMap::new();
        map.insert(0x0000, Cpu::f_0x0000);
        map.insert(0x1000, Cpu::f_0x1000);
        map.insert(0x2000, Cpu::f_0x2000);
        map.insert(0x3000, Cpu::f_0x3000);
        map.insert(0x4000, Cpu::f_0x4000);
        map.insert(0x5000, Cpu::f_0x5000);
        map.insert(0x6000, Cpu::f_0x6000);
        map.insert(0x7000, Cpu::f_0x7000);
        map.insert(0x8000, Cpu::f_0x8000);
        map.insert(0x9000, Cpu::f_0x9000);
        map.insert(0xA000, Cpu::f_0xA000);
        map.insert(0xB000, Cpu::f_0xB000);
        map.insert(0xC000, Cpu::f_0xC000);
        map.insert(0xD000, Cpu::f_0xD000);
        map.insert(0xE000, Cpu::f_0xE000);
        map.insert(0xF000, Cpu::f_0xF000);

        Cpu {
            memory: [0; 4096],
            stack: [0; 16],
            sp: 0,
            i: 0,
            pc: 0x200,
            v: [0; 16],
            graphics: [0; ROW * COL],
            keypad: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            opcode_function: map,
            should_redraw: false,
        }
    }

    pub fn load_font(&mut self) {
        for (i, &value) in FONTSET.iter().enumerate() {
            self.memory[i] = value;
        }
    }
}

impl Cpu {
    pub fn fetch_opcode(&mut self) -> u16 {
        let left: u16 = self.memory[self.pc as usize].into();
        let right: u16 = (self.memory[self.pc as usize + 1]).into();
        let opcode: u16 = (left << 8) | right;
        self.pc += 2;
        opcode
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            println!("some sound ...");
        }
    }
}

impl Cpu {
    pub fn decode_and_execute_graphic(&mut self, opcode: u16, graphics: Option<&mut Graphics>) {
        if opcode & 0xF000 == 0xF000 && opcode & 0x00FF == 0x0A {
            self.f_0xFX0A(opcode, &mut graphics.unwrap());
            return;
        }
        match self.opcode_function.get(&(opcode & 0xF000)) {
            Some(func) => func(self, opcode),
            None => {
                eprintln!("Opcode '{:#X}' not found", opcode);
            }
        }
    }
    pub fn decode_and_execute(&mut self, opcode: u16) {
        self.decode_and_execute_graphic(opcode, None);
    }
}

impl Cpu {
    pub fn f_0x0000(&mut self, opcode: u16) {
        match opcode & 0x000F {
            // 0x00E0: Clears the screen
            0x0000 => self.clear_screen(),

            // 0x00EE: Returns from subroutine
            0x000E => self.return_from_subroutine(),
            _ => eprintln!("Opcode '{:#X}' not found", opcode),
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

    // 0x1NNN: Jumps to address NNN
    fn f_0x1000(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;

        println!("Jumping to address {:#X}", address);

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
            0x9E => {
                // if key is pressed
                if self.keypad[((opcode & 0x0F00) >> 8) as usize] != 0 {
                    self.pc += 2;
                }
            }
            0xA1 => {
                // if key is not pressed
                if self.keypad[((opcode & 0x0F00) >> 8) as usize] == 0 {
                    self.pc += 2;
                }
            }
            _ => unreachable!(),
        }
    }

    fn f_0xF000(&mut self, opcode: u16) {
        match opcode & 0x00FF {
            0x0A => todo!(),
            _ => unreachable!(),
        }
    }

    // 0xFX07 sets v[X] to value of delay timer
    fn f_0xFX07(&mut self, opcode: u16) {
        let X = (opcode & 0x0F00) >> 8;
        self.v[X as usize] = self.delay_timer;
    }

    // 0xFX0A waits for keyboard input, and sets the value into v[X]
    fn f_0xFX0A(&mut self, opcode: u16, graphics: &mut Graphics) -> Option<usize> {
        let X = (opcode & 0x0F00) >> 8;

        while graphics.app.has_event() {
            match graphics.app.next_event() {
                simple::Event::Keyboard { is_down: true, key } => {
                    let pos = KEYMAP.iter().position(|&k| k == key)?;
                    self.keypad[X as usize] = pos as u8;
                    println!("Found key at pos {} -> {:?}", pos, KEYMAP[pos]);
                }
                _ => (),
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_screen() {
        let mut chip = Cpu::new();
        chip.graphics.fill(1);
        chip.decode_and_execute(0x00E0);
        assert_eq!(chip.graphics, [0; ROW * COL]);
    }

    #[test]
    fn clear_screen2() {
        let mut chip = Cpu::new();
        chip.graphics.fill(1);
        assert_eq!(chip.graphics, [1; ROW * COL]);
        chip.decode_and_execute(0xE0);
        assert_eq!(chip.graphics, [0; ROW * COL]);
    }

    #[test]
    fn jump_to_address_0x1NNN() {
        let mut chip = Cpu::new();
        chip.decode_and_execute(0x1080);
        assert_eq!(chip.pc, 128);
    }

    #[test]
    fn call_subroutine_0x2NNN() {
        let mut chip = Cpu::new();

        // call subroutine at address 128, and put current 512 on the stack
        chip.decode_and_execute(0x2080);
        assert_eq!(chip.pc, 128);

        // call subroutine at address 5, and put current address 128 on the stack
        chip.decode_and_execute(0x2005);
        assert_eq!(chip.pc, 5);

        assert_eq!(chip.stack[chip.sp as usize - 1], 128);
    }

    #[test]
    fn skip_if_equal_0x3XNN() {
        let mut chip = Cpu::new();
        assert_eq!(chip.pc, 0x200);
        chip.v[2] = 4;

        chip.decode_and_execute(0x3205);
        assert_eq!(chip.pc, 0x200);
        chip.decode_and_execute(0x3204);

        assert_eq!(chip.pc, 0x202);
    }
    #[test]
    fn skip_if_not_equal_0x4XNN() {
        let mut chip = Cpu::new();
        assert_eq!(chip.pc, 0x200);
        chip.v[2] = 4;
        chip.decode_and_execute(0x4204);
        assert_eq!(chip.pc, 0x200);

        chip.decode_and_execute(0x4205);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn skip_xy_0x5XY0() {
        let mut chip = Cpu::new();
        chip.v[1] = 10;
        chip.v[4] = 11;

        chip.decode_and_execute(0x5140);
        assert_eq!(chip.pc, 0x200);

        chip.v[4] = 10;
        chip.decode_and_execute(0x5140);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn set_value_0x6XNN() {
        let mut chip = Cpu::new();

        // set v[0] = 128
        chip.decode_and_execute(0x6080);
        assert_eq!(chip.v[0], 128);

        // set v[10] = 128
        chip.decode_and_execute(0x6a80);
        assert_eq!(chip.v[10], 128);

        println!("{:?}", chip.v);
    }

    #[test]
    fn add_to_value_0x7XNN() {
        let mut chip = Cpu::new();

        // set v[10] = 8
        chip.decode_and_execute(0x6a08);
        assert_eq!(chip.v[10], 8);

        chip.decode_and_execute(0x7a08);
        assert_eq!(chip.v[10], 16);
    }

    #[test]
    fn assign_value_0x8XY0() {
        let mut chip = Cpu::new();
        chip.v[3] = 4;
        chip.decode_and_execute(0x8430);
        assert_eq!(chip.v[4], 4);
    }

    #[test]
    fn or_value_0x8XY1() {
        let mut chip = Cpu::new();
        chip.v[8] = 10;
        chip.v[11] = 172;

        chip.decode_and_execute(0x88b1);

        assert_eq!(chip.v[8], 10 | 172);
    }

    #[test]
    fn and_value_0x8XY2() {
        let mut chip = Cpu::new();
        chip.v[8] = 10;
        chip.v[11] = 172;

        chip.decode_and_execute(0x88b2);

        assert_eq!(chip.v[8], 10 & 172);
    }

    #[test]
    fn xor_value_0x8XY3() {
        let mut chip = Cpu::new();
        chip.v[8] = 10;
        chip.v[11] = 172;

        chip.decode_and_execute(0x88b3);

        assert_eq!(chip.v[8], 10 ^ 172);
    }

    #[test]
    fn adding_registers_simple_0x8XY4() {
        let mut chip = Cpu::new();

        chip.v[2] = 5;
        chip.v[3] = 10;

        chip.decode_and_execute(0x8234);

        assert_eq!(chip.v[2], 15);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn adding_registers_with_carry_0x8XY4() {
        let mut chip = Cpu::new();

        chip.v[2] = 255;
        chip.v[3] = 1;

        chip.decode_and_execute(0x8234);

        assert_eq!(chip.v[2], 0x1);
        assert_eq!(chip.v[0xF], 0x1);
    }

    #[test]
    fn subtracting_registers_simple_0x8XY5() {
        let mut chip = Cpu::new();

        chip.v[2] = 10;
        chip.v[3] = 5;

        chip.decode_and_execute(0x8235);

        assert_eq!(chip.v[2], 5);
        assert_eq!(chip.v[0xF], 0x1);
    }

    #[test]
    fn subtracting_registers_with_borrow_0x8XY5() {
        let mut chip = Cpu::new();

        chip.v[2] = 5;
        chip.v[3] = 10;

        chip.decode_and_execute(0x8235);

        assert_eq!(chip.v[2], 251);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn right_shifting_0x8XY6() {
        let mut chip = Cpu::new();

        chip.v[2] = 3;
        chip.v[3] = 5;
        chip.decode_and_execute(0x8326);
        assert_eq!(chip.v[3], 0x1);
        assert_eq!(chip.v[0xF], 0x1);

        chip.v[2] = 3;
        chip.v[3] = 4;
        chip.decode_and_execute(0x8326);
        assert_eq!(chip.v[3], 0x1);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn subtracting_y_x_simple_0x8XY7() {
        let mut chip = Cpu::new();

        chip.v[2] = 3;
        chip.v[3] = 1;

        chip.decode_and_execute(0x8327);

        assert_eq!(chip.v[3], 2);
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn subtracting_y_x_with_borrow_0x8XY7() {
        let mut chip = Cpu::new();

        chip.v[2] = 3;
        chip.v[3] = 10;

        chip.decode_and_execute(0x8327);

        assert_eq!(chip.v[3], 249);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn left_shifting_0x8XYE() {
        let mut chip = Cpu::new();

        chip.v[3] = 4; //X
        chip.v[2] = 128; //Y

        chip.decode_and_execute(0x832E);

        assert_eq!(chip.v[0xF], 0x1);
        assert_eq!(chip.v[3], 0x0);
    }

    #[test]
    fn skip_if_xy_not_equal_0x9000() {
        let mut chip = Cpu::new();

        chip.v[2] = 5;
        chip.v[3] = 5;
        chip.decode_and_execute(0x9230);

        assert_eq!(chip.pc, 0x200);

        chip.v[3] = 6;
        chip.decode_and_execute(0x9230);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn set_index_register_value_0xA000() {
        let mut chip = Cpu::new();

        chip.decode_and_execute(0xA001);
        assert_eq!(chip.i, 1);

        chip.decode_and_execute(0xA123);
        assert_eq!(chip.i, 291);
    }

    #[test]
    fn jump_to_nnn_plus_v0_0xB000() {
        let mut chip = Cpu::new();
        assert_eq!(chip.pc, 0x200);
        chip.v[0] = 0x80;
        chip.decode_and_execute(0xB080);
        assert_eq!(chip.pc, 0x80 + 0x80);
    }

    #[test]
    fn is_key_pressed_0xE000() {
        let mut chip = Cpu::new();
        assert_eq!(chip.pc, 0x200);

        chip.keypad[3] = 1;
        chip.decode_and_execute(0xE39E);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn is_key_not_pressed_0xE000() {
        let mut chip = Cpu::new();
        assert_eq!(chip.pc, 0x200);

        chip.keypad[3] = 1;
        chip.decode_and_execute(0xE3A1);
        assert_eq!(chip.pc, 0x200);

        chip.keypad[3] = 0;
        chip.decode_and_execute(0xE3A1);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn set_delay_timer_0xFX07() {
        let mut chip = Cpu::new();
        chip.delay_timer = 10;

        chip.decode_and_execute(0xFA07);

        assert_eq!(chip.v[10], 10);
    }
}
