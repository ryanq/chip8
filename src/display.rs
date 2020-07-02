use {
    crate::Error,
    log::*,
    sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl},
    std::fmt::{self, Formatter},
};

pub struct Display {
    w: usize,
    h: usize,
    scale: usize,
    pixels: Vec<u8>,
    canvas: Canvas<Window>,
    dirty: bool,
}

impl Display {
    pub fn new(sdl: &Sdl, gui_scale: u32, width: u32, height: u32) -> Result<Display, Error> {
        let (w, h) = (width as usize, height as usize);
        let scale = gui_scale as usize;

        info!(target: "sdl", "creating window at {}x scale ({}x{} pixels)", scale, w * scale, h * scale);
        let video = sdl.video()?;
        let window = video
            .window("CHIP-8", width * gui_scale, height * gui_scale)
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().build()?;

        Ok(Display {
            w,
            h,
            scale,
            pixels: vec![0; w * h],
            canvas,
            dirty: true,
        })
    }

    pub fn needs_presenting(&self) -> bool {
        self.dirty
    }

    pub fn clear_screen(&mut self) -> Result<(), Error> {
        debug!(target: "dsp", "clearing screen backing buffer");
        for pixel in self.pixels.iter_mut() {
            *pixel = 0;
        }

        self.dirty = true;
        Ok(())
    }

    pub fn draw_sprite(&mut self, sprite: &[u8], x: u8, y: u8) -> Result<bool, Error> {
        debug!(target: "dsp", "drawing sprite to backing buffer");
        if log_enabled!(target: "dsp", Level::Trace) {
            let mut chunks = sprite.chunks_exact(2);
            while let Some(&[one, two]) = chunks.next() {
                let mut one = one.reverse_bits();
                let mut two = two.reverse_bits();
                for _ in 0..8 {
                    match (one & 1 != 0, two & 1 != 0) {
                        (false, false) => print!(" "),
                        (false, true) => print!("▄"),
                        (true, false) => print!("▀"),
                        (true, true) => print!("█"),
                    }
                    one >>= 1;
                    two >>= 1;
                }
                println!("│");
            }
            for byte in chunks.remainder() {
                let mut byte = byte.reverse_bits();
                for _ in 0..8 {
                    match byte & 1 != 0 {
                        false => print!(" "),
                        true => print!("▀"),
                    }
                    byte >>= 1;
                }
                println!("│");
            }
        }

        let x = x as usize;
        let y = y as usize;
        let mut toggled_off = false;

        for dy in 0..sprite.len() {
            let mut byte = sprite[dy].reverse_bits();
            for dx in 0..8 {
                if byte & 1 != 0 {
                    let index = (y + dy) * self.w + (x + dx);
                    match self.pixels[index] {
                        0 => self.pixels[index] = 1,
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

        self.dirty = true;
        Ok(toggled_off)
    }

    pub fn present(&mut self) -> Result<(), String> {
        debug!(target: "sdl", "updating canvas");
        let scale = self.scale as f32;
        self.canvas.set_scale(scale, scale)?;

        trace!(target: "sdl", "clearing canvas");
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        self.canvas.set_draw_color(Color::WHITE);
        for y in 0..self.h {
            for x in 0..self.w {
                let index = y * self.w + x;
                if self.pixels[index] != 0 {
                    trace!(target: "sdl", "drawing pixel ({}, {})", x, y);
                    let pixel = Rect::new(x as i32, y as i32, 1, 1);
                    self.canvas.fill_rect(pixel)?;
                }
            }
        }

        debug!(target: "sdl", "presenting canvas");
        self.canvas.present();

        self.dirty = false;
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
