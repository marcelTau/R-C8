#![allow(dead_code)]

mod chip8;
mod cpu;
mod graphics;

use chip8::Chip8;

fn main() {
    let mut chip = Chip8::new();

    chip.load_program("IBM.ch8").unwrap_or_else(|err| {
        eprintln!("Error occured during loading the program: {}", err);
        std::process::exit(1);
    });
    chip.gameloop();

    //chip.setup_map();

    //chip.load_game("IBM.ch8").unwrap_or_else(|err| {
    //eprintln!("Error occured during loading the program: {}", err);
    //std::process::exit(1);
    //});

    //chip.load_font();

    //chip.run();
}
