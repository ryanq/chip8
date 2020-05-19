use {
    crate::Error,
    sdl2::{event::Event, keyboard::Keycode, EventPump, Sdl},
    std::{thread, time::Duration},
};

pub struct Input {
    events: EventPump,
    key_status: [bool; 16],
    last_key: Option<u8>,
    pub quit: bool,
}

impl Input {
    pub fn new(sdl: &Sdl) -> Result<Input, Error> {
        let events = sdl.event_pump()?;

        Ok(Input {
            events,
            key_status: [false; 16],
            last_key: None,
            quit: false,
        })
    }

    pub fn handle_input(&mut self) {
        let events = self.events.poll_iter().collect::<Vec<_>>();
        for event in events {
            match event {
                Event::Quit { .. } => self.quit = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    repeat: false,
                    ..
                } => self.key_down(0x1),
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    repeat: false,
                    ..
                } => self.key_up(0x1),
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    repeat: false,
                    ..
                } => self.key_down(0x2),
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    repeat: false,
                    ..
                } => self.key_up(0x2),
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    repeat: false,
                    ..
                } => self.key_down(0x3),
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    repeat: false,
                    ..
                } => self.key_up(0x3),
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    repeat: false,
                    ..
                } => self.key_down(0xc),
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    repeat: false,
                    ..
                } => self.key_up(0xc),
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    repeat: false,
                    ..
                } => self.key_down(0x4),
                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    repeat: false,
                    ..
                } => self.key_up(0x4),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    repeat: false,
                    ..
                } => self.key_down(0x5),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    repeat: false,
                    ..
                } => self.key_up(0x5),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    repeat: false,
                    ..
                } => self.key_down(0x6),
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    repeat: false,
                    ..
                } => self.key_up(0x6),
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => self.key_down(0xd),
                Event::KeyUp {
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => self.key_up(0xd),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    repeat: false,
                    ..
                } => self.key_down(0x7),
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    repeat: false,
                    ..
                } => self.key_up(0x7),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => self.key_down(0x8),
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => self.key_up(0x8),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    repeat: false,
                    ..
                } => self.key_down(0x9),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    repeat: false,
                    ..
                } => self.key_up(0x9),
                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    repeat: false,
                    ..
                } => self.key_down(0xe),
                Event::KeyUp {
                    keycode: Some(Keycode::T),
                    repeat: false,
                    ..
                } => self.key_up(0xe),
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    repeat: false,
                    ..
                } => self.key_down(0xa),
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    repeat: false,
                    ..
                } => self.key_up(0xa),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    repeat: false,
                    ..
                } => self.key_down(0x0),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    repeat: false,
                    ..
                } => self.key_up(0x0),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    repeat: false,
                    ..
                } => self.key_down(0xb),
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    repeat: false,
                    ..
                } => self.key_up(0xb),
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    repeat: false,
                    ..
                } => self.key_down(0xf),
                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    repeat: false,
                    ..
                } => self.key_up(0xf),
                _ => (),
            }
        }
    }

    pub fn wait_for_input(&mut self) -> u8 {
        self.last_key = None;

        while self.last_key.is_none() {
            self.handle_input();

            thread::sleep(Duration::from_millis(5));
        }

        // SAFETY: The exit condition of the loop above is that the last_key
        //         field is not None.
        self.last_key.unwrap()
    }
}

impl Input {
    fn key_down(&mut self, value: u8) {
        self.key_status[value as usize] = true;
        self.last_key = Some(value);
    }

    fn key_up(&mut self, value: u8) {
        self.key_status[value as usize] = false;
    }
}
