use log::{debug, trace};
use quark::BitIndex;
use std::iter;

use crate::display::Display;

pub const PROGRAM_START: usize = 0x200;
const FONT_DATA_START: usize = 0x0;
const FONT_DIGIT_SIZE: usize = 5;
pub const SCREEN_WIDTH_PIXELS: usize = 64;
pub const SCREEN_HEIGHT_PIXELS: usize = 32;

pub struct Chip8 {
    v: [u8; 16],
    i: usize,
    pc: usize,
    memory: Vec<u8>,
    halted: bool,
}

impl Default for Chip8 {
    fn default() -> Chip8 {
        Chip8 {
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            memory: vec![0; 0x1000],
            halted: false,
        }
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c8 = Chip8::default();
        c8.load_font_data_at(FONT_DATA_START);
        c8
    }

    pub fn load_at(&mut self, start: usize, source: impl IntoIterator<Item = u8>) {
        self.memory.truncate(start);
        self.memory.extend(
            source
                .into_iter()
                .chain(iter::repeat(0))
                .take(0x1000 - start),
        );
    }

    pub fn step(&mut self, display: &mut Display) {
        if !self.halted {
            self.step1(display);
        }
    }
}

impl Chip8 {
    fn step1(&mut self, display: &mut Display) {
        let opcode = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        match (
            opcode.bits(12..16),
            opcode.bits(8..12),
            opcode.bits(4..8),
            opcode.bits(0..4),
        ) {
            (0x0, 0x0, 0xe, 0x0) => {
                debug!("{:03x}: [{:04x}]  clearing the screen", self.pc, opcode);
                display.clear_screen();
            }
            (0x6, ..) => {
                let x = opcode.bits(8..12) as usize;
                let value = opcode.bits(0..8) as u8;
                debug!(
                    "{:03x}: [{:04x}]  assign {:02x}h to V{:1x}",
                    self.pc, opcode, value, x
                );
                self.v[x] = value;
            }
            (0x7, ..) => {
                let x = opcode.bits(8..12) as usize;
                let value = opcode.bits(0..8) as u8;
                debug!(
                    "{:03x}: [{:04x}]  add the value {:02x}h to V{:1x}",
                    self.pc, opcode, value, x
                );
                let value = self.v[x] as u16 + value as u16;
                self.v[x] = (value % 256) as u8;
            }
            (0xa, ..) => {
                let address = opcode.bits(0..12) as usize;
                debug!(
                    "{:03x}: [{:04x}]  assign {:03x}h to I",
                    self.pc, opcode, address
                );
                self.i = address;
            }
            (0xc, ..) => {
                let x = opcode.bits(8..12) as usize;
                let mask = opcode.bits(0..8) as u8;
                debug!(
                    "{:03x}: [{:04x}]  assign a random byte (masked by {:02x}h) to V{:1x}",
                    self.pc, opcode, mask, x
                );
                let byte: u8 = rand::random();
                self.v[x] = byte & mask;
            }
            (0xd, ..) => {
                let vx = opcode.bits(8..12) as usize;
                let vy = opcode.bits(4..8) as usize;
                let n = opcode.bits(0..4) as usize;
                debug!(
                    "{:03x}: [{:04x}]  draw {} byte sprite to the screen at V{:1x}, V{:1x}",
                    self.pc, opcode, n, vx, vy
                );
                let sprite = self.memory.get(self.i..(self.i + n));
                let x = self.v[vx];
                let y = self.v[vy];
                let toggled_off = display.draw_sprite(sprite.unwrap(), x, y);
                if toggled_off {
                    self.v[15] = 1;
                } else {
                    self.v[15] = 0;
                }
            }
            (0xf, _, 0x2, 0x9) => {
                let x = opcode.bits(8..12) as usize;
                debug!(
                    "{:03x}: [{:04x}]  assign address of digit in V{:1x} ({}) to I",
                    self.pc, opcode, x, self.v[x]
                );
                let digit = self.v[x] as usize;
                self.i = FONT_DATA_START + digit * FONT_DIGIT_SIZE;
            }
            (0xf, _, 0x3, 0x3) => {
                let x = opcode.bits(8..12) as usize;
                debug!(
                    "{:03x}: [{:04x}]  assign BCD of V{:1x} ({:02x}h) to {:03x}h, {:03x}h, {:03x}h",
                    self.pc,
                    opcode,
                    x,
                    self.v[x as usize],
                    self.i,
                    self.i + 1,
                    self.i + 2
                );
                let mut value = self.v[x as usize];
                let ones = value % 10;
                value /= 10;
                let tens = value % 10;
                value /= 10;
                let hundreds = value;

                trace!("digits: {}, {}, {}", hundreds, tens, ones);
                self.memory[self.i] = hundreds;
                self.memory[self.i + 1] = tens;
                self.memory[self.i + 2] = ones;
            }
            (0xf, _, 0x6, 0x5) => {
                let x = opcode.bits(8..12) as usize;
                debug!(
                    "{:03x}: [{:04x}]  assign values starting at {:03x}h to V0..=V{:1x}",
                    self.pc, opcode, self.i, x
                );
                for i in 0..=x {
                    let value = self.memory[self.i];
                    trace!("assigning {} (from {:03x}h) to V{}", value, self.i, i);
                    self.v[i] = value;
                    self.i += 1;
                }
            }
            _ => {
                debug!("{:03x}: [{:04x}]  halting", self.pc, opcode);
                self.halted = true;
                return;
            }
        }

        self.pc += 2;
    }

    fn load_font_data_at(&mut self, start: usize) {
        // digit 0
        self.memory[start + 0] = 0xf0;
        self.memory[start + 1] = 0x90;
        self.memory[start + 2] = 0x90;
        self.memory[start + 3] = 0x90;
        self.memory[start + 4] = 0xf0;

        // digit 1
        self.memory[start + 5] = 0x20;
        self.memory[start + 6] = 0x60;
        self.memory[start + 7] = 0x20;
        self.memory[start + 8] = 0x20;
        self.memory[start + 9] = 0x70;

        // digit 2
        self.memory[start + 10] = 0xf0;
        self.memory[start + 11] = 0x10;
        self.memory[start + 12] = 0xf0;
        self.memory[start + 13] = 0x80;
        self.memory[start + 14] = 0xf0;

        // digit 3
        self.memory[start + 15] = 0xf0;
        self.memory[start + 16] = 0x10;
        self.memory[start + 17] = 0xf0;
        self.memory[start + 18] = 0x10;
        self.memory[start + 19] = 0xf0;

        // digit 4
        self.memory[start + 20] = 0x90;
        self.memory[start + 21] = 0x90;
        self.memory[start + 22] = 0xf0;
        self.memory[start + 23] = 0x10;
        self.memory[start + 24] = 0x10;

        // digit 5
        self.memory[start + 25] = 0xf0;
        self.memory[start + 26] = 0x80;
        self.memory[start + 27] = 0xf0;
        self.memory[start + 28] = 0x10;
        self.memory[start + 29] = 0xf0;

        // digit 6
        self.memory[start + 30] = 0xf0;
        self.memory[start + 31] = 0x80;
        self.memory[start + 32] = 0xf0;
        self.memory[start + 33] = 0x90;
        self.memory[start + 34] = 0xf0;

        // digit 7
        self.memory[start + 35] = 0xf0;
        self.memory[start + 36] = 0x10;
        self.memory[start + 37] = 0x20;
        self.memory[start + 38] = 0x40;
        self.memory[start + 39] = 0x40;

        // digit 8
        self.memory[start + 40] = 0xf0;
        self.memory[start + 41] = 0x90;
        self.memory[start + 42] = 0xf0;
        self.memory[start + 43] = 0x90;
        self.memory[start + 44] = 0xf0;

        // digit 9
        self.memory[start + 45] = 0xf0;
        self.memory[start + 46] = 0x90;
        self.memory[start + 47] = 0xf0;
        self.memory[start + 48] = 0x10;
        self.memory[start + 49] = 0xf0;

        // digit A
        self.memory[start + 50] = 0xf0;
        self.memory[start + 51] = 0x90;
        self.memory[start + 52] = 0xf0;
        self.memory[start + 53] = 0x90;
        self.memory[start + 54] = 0x90;

        // digit B
        self.memory[start + 55] = 0xe0;
        self.memory[start + 56] = 0x90;
        self.memory[start + 57] = 0xe0;
        self.memory[start + 58] = 0x90;
        self.memory[start + 59] = 0xe0;

        // digit C
        self.memory[start + 60] = 0xf0;
        self.memory[start + 61] = 0x80;
        self.memory[start + 62] = 0x80;
        self.memory[start + 63] = 0x80;
        self.memory[start + 64] = 0xf0;

        // digit D
        self.memory[start + 65] = 0xe0;
        self.memory[start + 66] = 0x90;
        self.memory[start + 67] = 0x90;
        self.memory[start + 68] = 0x90;
        self.memory[start + 69] = 0xe0;

        // digit E
        self.memory[start + 70] = 0xf0;
        self.memory[start + 71] = 0x80;
        self.memory[start + 72] = 0xf0;
        self.memory[start + 73] = 0x80;
        self.memory[start + 74] = 0xf0;

        // digit F
        self.memory[start + 75] = 0xf0;
        self.memory[start + 76] = 0x80;
        self.memory[start + 77] = 0xf0;
        self.memory[start + 78] = 0x80;
        self.memory[start + 79] = 0x80;
    }
}
