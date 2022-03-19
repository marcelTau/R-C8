#![allow(dead_code)]

mod lib;

use rc8::Chip8;

fn main() {
    let mut chip = Chip8::new();

    chip.load_game("pong.rom").unwrap_or_else(|err| {
        eprintln!("Error occured during loading the program: {}", err);
        std::process::exit(1);
    });

    chip.load_font();

    chip.run();
}
