use {
    crate::{audio::Audio, cli::*, display::Display, input::Input, Error},
    log::*,
    quark::BitIndex,
    std::fs::File,
    std::io::Read,
    std::thread,
    std::time::Duration,
};

const PROGRAM_START: usize = 0x200;
const STACK_START: usize = PROGRAM_START - 32;

pub struct Chip8 {
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    at: u8,
    dt: u8,
    memory: Vec<u8>,
    audio: Audio,
    display: Display,
    input: Input,
    cycles: u64,
    halted: bool,
}

const CYCLES_PER_SECOND: u64 = 120;
const CYCLE_RATE: Duration = Duration::from_nanos(1_000_000_000 / CYCLES_PER_SECOND);

impl Chip8 {
    pub fn new(config: &Config) -> Result<Chip8, Error> {
        let sdl = sdl2::init()?;

        let audio = Audio::new(&sdl)?;
        let display = Display::new(&sdl, &config)?;
        let input = Input::new(&sdl, &config)?;

        let mut memory = vec![0; 0x1000];
        memory[0..][..FONT_DATA.len()].copy_from_slice(FONT_DATA);

        let program = {
            let mut file = File::open(&config.program)?;
            let mut buffer = Vec::with_capacity(0x1000);
            let size = file.read_to_end(&mut buffer)?;
            info!(target: "cli", "read {} bytes from {}", size, config.program.display());

            buffer
        };
        memory[PROGRAM_START..][..program.len()].copy_from_slice(&program);

        Ok(Chip8 {
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            sp: STACK_START,
            at: 0,
            dt: 0,
            memory,
            audio,
            display,
            input,
            cycles: 0,
            halted: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        info!(target: "exe", "starting run loop");
        self.display.present()?;
        loop {
            self.input.process_pending_input();
            if self.input.quit {
                info!(target: "exe", "quit requested; halting");
                break;
            }

            self.step()?;
            self.cycles += 1;

            self.update_timers();

            if self.display.needs_presenting() {
                self.display.present()?;
            }

            if self.at == 0 {
                self.audio.stop();
            } else {
                self.audio.start();
            }

            thread::sleep(CYCLE_RATE);
        }

        Ok(())
    }
}

impl Chip8 {
    fn step(&mut self) -> Result<(), Error> {
        if self.halted {
            return Ok(());
        }

        let pc = self.pc;
        self.pc += 2;

        let opcode = u16::from_be_bytes([self.memory[pc], self.memory[pc + 1]]);
        match (
            opcode.bits(12..16),
            opcode.bits(8..12),
            opcode.bits(4..8),
            opcode.bits(0..4),
        ) {
            (0x0, 0x0, 0xe, 0x0) => {
                debug!(target: "asm", "{:03x}: [{:04x}] cls", pc, opcode);
                self.display.clear_screen()?;
            }
            (0x0, 0x0, 0xe, 0xe) => {
                debug!(target: "asm", "{:03x}: [{:04x}] ret", pc, opcode);
                let address = u16::from_be_bytes([self.memory[self.sp], self.memory[self.sp + 1]]);
                self.sp -= 2;
                self.pc = address as usize;
            }
            (0x0, ..) => {
                let address = opcode.bits(0..12);
                error!(target: "asm", "{:03x}: [{:04x}] sys {:03x}", pc, opcode, address);
                self.halted = true;
                return Ok(());
            }
            (0x1, ..) => {
                let address = opcode.bits(0..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] jp {:03x}", pc, opcode, address);
                self.pc = address;
                return Ok(());
            }
            (0x2, ..) => {
                let address = opcode.bits(0..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] call {:03x}", pc, opcode, address);
                self.sp += 2;
                let bytes = (self.pc as u16).to_be_bytes();
                self.memory[self.sp] = bytes[0];
                self.memory[self.sp + 1] = bytes[1];
                self.pc = address;
            }
            (0x3, ..) => {
                let x = opcode.bits(8..12) as usize;
                let value = opcode.bits(0..8) as u8;
                debug!(target: "asm", "{:03x}: [{:04x}] se v{:1x}, {:02x}", pc, opcode, x, value);
                if self.v[x] == value {
                    self.pc += 2;
                }
            }
            (0x4, ..) => {
                let x = opcode.bits(8..12) as usize;
                let value = opcode.bits(0..8) as u8;
                debug!(target: "asm", "{:03x}: [{:04x}] sne v{:1x}, {:02x}", pc, opcode, x, value);
                if self.v[x] != value {
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0x0) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] se v{:1x}, v{:1x}", pc, opcode, x, y);
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            (0x6, ..) => {
                let x = opcode.bits(8..12) as usize;
                let value = opcode.bits(0..8) as u8;
                debug!(target: "asm", "{:03x}: [{:04x}] ld v{:1x}, {:02x}", pc, opcode, x, value);
                self.v[x] = value;
            }
            (0x7, ..) => {
                let x = opcode.bits(8..12) as usize;
                let value = opcode.bits(0..8) as u8;
                debug!(target: "asm", "{:03x}: [{:04x}] add v{:1x}, {:02x}", pc, opcode, x, value);
                let value = self.v[x] as u16 + value as u16;
                self.v[x] = (value % 256) as u8;
            }
            (0x8, _, _, 0x0) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld v{:1x}, v{:1x}", pc, opcode, x, y);
                self.v[x] = self.v[y];
            }
            (0x8, _, _, 0x1) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] or v{:1x}, v{:1x}", pc, opcode, x, y);
                self.v[x] = self.v[x] | self.v[y];
            }
            (0x8, _, _, 0x2) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] and v{:1x}, v{:1x}", pc, opcode, x, y);
                self.v[x] = self.v[x] & self.v[y];
            }
            (0x8, _, _, 0x3) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] xor v{:1x}, v{:1x}", pc, opcode, x, y);
                self.v[x] = self.v[x] ^ self.v[y];
            }
            (0x8, _, _, 0x4) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] add v{:1x}, v{:1x}", pc, opcode, x, y);
                let (value, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = value;
                self.v[15] = if overflow { 1 } else { 0 };
            }
            (0x8, _, _, 0x5) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] sub v{:1x}, v{:1x}", pc, opcode, x, y);
                let (value, borrow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = value;
                self.v[15] = if !borrow { 1 } else { 0 };
            }
            (0x8, _, _, 0x6) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] shr v{:1x}, v{:1x}", pc, opcode, x, y);
                self.v[15] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            (0x8, _, _, 0x7) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] subn v{:1x}, v{:1x}", pc, opcode, x, y);
                let (value, borrow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = value;
                self.v[15] = if !borrow { 1 } else { 0 };
            }
            (0x8, _, _, 0xe) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] shl v{:1x}, v{:1x}", pc, opcode, x, y);
                self.v[15] = self.v[x] & 0x80;
                self.v[x] <<= 1;
            }
            (0x9, _, _, 0x0) => {
                let x = opcode.bits(8..12) as usize;
                let y = opcode.bits(4..8) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] sne v{:1x}, v{:1x}", pc, opcode, x, y);
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            (0xa, ..) => {
                let address = opcode.bits(0..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld i, {:03x}", pc, opcode, address);
                self.i = address;
            }
            (0xb, ..) => {
                let address = opcode.bits(0..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] jp v0, {:03x}", pc, opcode, address);
                let address = self.v[0] as usize + address;
                self.pc = address;
            }
            (0xc, ..) => {
                let x = opcode.bits(8..12) as usize;
                let mask = opcode.bits(0..8) as u8;
                debug!(target: "asm", "{:03x}: [{:04x}] rnd v{:1x}, {:02x}", pc, opcode, x, mask);
                let byte: u8 = rand::random();
                self.v[x] = byte & mask;
            }
            (0xd, ..) => {
                let vx = opcode.bits(8..12) as usize;
                let vy = opcode.bits(4..8) as usize;
                let n = opcode.bits(0..4) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] drw v{:1x}, v{:1x}, {:1x}", pc, opcode, vx, vy, n);
                let sprite = &self.memory[self.i..][..n];
                let x = self.v[vx];
                let y = self.v[vy];
                let toggled_off = self.display.draw_sprite(sprite, x, y)?;
                if toggled_off {
                    self.v[15] = 1;
                } else {
                    self.v[15] = 0;
                }
            }
            (0xe, _, 0x9, 0xe) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] skp v{:1x}", pc, opcode, x);
                if self.input.is_key_pressed(self.v[x]) {
                    self.pc += 2;
                }
            }
            (0xe, _, 0xa, 0x1) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] sknp v{:1x}", pc, opcode, x);
                if !self.input.is_key_pressed(self.v[x]) {
                    self.pc += 2;
                }
            }
            (0xf, _, 0x0, 0x7) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld v{:1x}, dt", pc, opcode, x);
                self.v[x] = self.dt;
            }
            (0xf, _, 0x0, 0xa) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld v{:1x}, k", pc, opcode, x);
                let value = self.input.wait_for_input();
                self.v[x] = value;
            }
            (0xf, _, 0x1, 0x5) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld dt, v{:1x}", pc, opcode, x);
                self.dt = self.v[x];
            }
            (0xf, _, 0x1, 0x8) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld st, v{:1x}", pc, opcode, x);
                self.at = self.v[x];
            }
            (0xf, _, 0x1, 0xe) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] add i, v{:1x}", pc, opcode, x);
                self.i = self.i + self.v[x] as usize;
            }
            (0xf, _, 0x2, 0x9) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld f, v{:1x}", pc, opcode, x);
                let digit = self.v[x] as usize;
                self.i = FONT_DATA_START + digit * FONT_DIGIT_SIZE;
            }
            (0xf, _, 0x3, 0x3) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld b, v{:1x}", pc, opcode, x);
                let mut value = self.v[x as usize];
                let ones = value % 10;
                value /= 10;
                let tens = value % 10;
                value /= 10;
                let hundreds = value;

                self.memory[self.i] = hundreds;
                self.memory[self.i + 1] = tens;
                self.memory[self.i + 2] = ones;
            }
            (0xf, _, 0x5, 0x5) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld [i], v{:1x}", pc, opcode, x);
                for i in 0..=x {
                    let value = self.v[i];
                    self.memory[self.i] = value;
                    self.i += 1;
                }
            }
            (0xf, _, 0x6, 0x5) => {
                let x = opcode.bits(8..12) as usize;
                debug!(target: "asm", "{:03x}: [{:04x}] ld v{:1x}, [i]", pc, opcode, x);
                for i in 0..=x {
                    let value = self.memory[self.i];
                    self.v[i] = value;
                    self.i += 1;
                }
            }
            _ => {
                error!(target: "asm", "{:03x}: [{:04x}] unknown instruction", pc, opcode);
                self.halted = true;
                return Ok(());
            }
        }

        Ok(())
    }

    fn update_timers(&mut self) {
        if self.cycles < CYCLES_PER_SECOND / 60 {
            return;
        }
        self.cycles = 0;

        if self.at > 0 {
            self.at -= 1;
        }

        if self.dt > 0 {
            self.dt -= 1;
        }
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
