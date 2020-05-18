use log::trace;
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};
use std::fmt::{self, Formatter};

pub struct Display {
    pixels: Vec<u32>,
    w: usize,
}

impl Display {
    pub fn with_resolution(w: usize, h: usize) -> Display {
        Display {
            pixels: vec![0; w * h],
            w,
        }
    }

    pub fn clear_screen(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = 0;
        }
    }

    pub fn draw_sprite(&mut self, sprite: &[u8], x: u8, y: u8) -> bool {
        let x = x as usize;
        let y = y as usize;
        let mut toggled_off = false;

        trace!("drawing sprite {:?}", sprite);

        for dy in 0..sprite.len() {
            let mut byte = sprite[dy].reverse_bits();
            for dx in 0..8 {
                if byte & 1 != 0 {
                    let index = (y + dy) * self.w + (x + dx);
                    match self.pixels[index] {
                        0 => self.pixels[index] = 0x00ff_ffff,
                        1 => {
                            self.pixels[index] = 0;
                            toggled_off = true;
                        }
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    }
                }
                byte >>= 1;
            }
        }

        toggled_off
    }

    pub fn update_screen(&self, screen: &mut Canvas<Window>, scale: u32) -> Result<(), String> {
        screen.set_scale(scale as f32, scale as f32)?;
        screen.set_draw_color(Color::WHITE);

        let height = self.pixels.len() / self.w;
        for y in 0..height {
            for x in 0..self.w {
                let index = y * self.w + x;
                if self.pixels[index] != 0 {
                    let pixel = Rect::new(x as i32, y as i32, 1, 1);
                    screen.fill_rect(pixel)?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for rows in self.pixels.chunks_exact(self.w * 2) {
            let first = &rows[..self.w];
            let second = &rows[self.w..];
            for pair in first.iter().zip(second.iter()) {
                match pair {
                    (0, 0) => write!(f, " ")?,
                    (0, _) => write!(f, "▄")?,
                    (_, 0) => write!(f, "▀")?,
                    (_, _) => write!(f, "█")?,
                }
            }
            println!();
        }

        Ok(())
    }
}
