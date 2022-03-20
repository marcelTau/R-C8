use crate::chip8::{COL, ROW};

pub struct Graphics {
    pub app: simple::Window,
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            app: simple::Window::new("Chip8", (COL * 10) as u16, (ROW * 10) as u16),
        }
    }
    pub fn draw(&mut self, map: &[u8; ROW * COL]) {
        for (i, &value) in map.iter().enumerate() {
            if value == 0 {
                continue;
            }
            let x = i % COL;
            let y = i / ROW;

            let r = simple::Point::new(x as i32, y as i32);
            self.app.draw_point(r);
            self.app.set_color(255, 255, 255, 255);
        }
    }
}
