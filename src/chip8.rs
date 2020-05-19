use {
    crate::{display::Display, Error},
    log::{debug, trace},
    quark::BitIndex,
};

const PROGRAM_START: usize = 0x200;
pub const SCREEN_WIDTH_PIXELS: u32 = 64;
pub const SCREEN_HEIGHT_PIXELS: u32 = 32;

pub struct Chip8 {
    v: [u8; 16],
    i: usize,
    pc: usize,
    memory: Vec<u8>,
    display: Display,
    halted: bool,
}

impl Chip8 {
    pub fn new(program: &[u8], gui_scale: u32) -> Result<Chip8, Error> {
        let sdl = sdl2::init()?;

        let display = Display::new(sdl, gui_scale, SCREEN_WIDTH_PIXELS, SCREEN_HEIGHT_PIXELS)?;

        let mut memory = vec![0; 0x1000];
        memory[0..][..FONT_DATA.len()].copy_from_slice(FONT_DATA);
        memory[PROGRAM_START..][..program.len()].copy_from_slice(program);

        Ok(Chip8 {
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            memory,
            display,
            halted: false,
        })
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.halted {
            return Ok(());
        }

        let opcode = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        match (
            opcode.bits(12..16),
            opcode.bits(8..12),
            opcode.bits(4..8),
            opcode.bits(0..4),
        ) {
            (0x0, 0x0, 0xe, 0x0) => {
                debug!("{:03x}: [{:04x}]  clearing the screen", self.pc, opcode);
                self.display.clear_screen();
                self.display.present()?;
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
                let sprite = &self.memory[self.i..][..n];
                let x = self.v[vx];
                let y = self.v[vy];
                let toggled_off = self.display.draw_sprite(sprite, x, y);
                if toggled_off {
                    self.v[15] = 1;
                } else {
                    self.v[15] = 0;
                }
                self.display.present()?;
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
                return Ok(());
            }
        }

        self.pc += 2;

        Ok(())
    }
}

static FONT_DATA: &[u8] = &[
    0xf0, 0x90, 0x90, 0x90, 0xf0, // digit 0
    0x20, 0x60, 0x20, 0x20, 0x70, // digit 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // digit 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // digit 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // digit 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // digit 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // digit 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // digit 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // digit 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // digit 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // digit A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // digit B
    0xf0, 0x80, 0x80, 0x80, 0xf0, // digit C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // digit D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // digit E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // digit F
];
const FONT_DATA_START: usize = 0x0;
const FONT_DIGIT_SIZE: usize = 5;
