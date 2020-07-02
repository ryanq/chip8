use {
    crate::Error,
    log::*,
    sdl2::{event::Event, keyboard::{Keycode, Mod}, EventPump, Sdl},
    std::{collections::HashMap, thread, time::Duration},
};

pub struct Input {
    events: EventPump,
    key_map: HashMap<Keycode, u8>,
    key_status: [bool; 16],
    last_key: Option<u8>,
    pub quit: bool,
}

impl Input {
    pub fn new(sdl: &Sdl, keymap: &str) -> Result<Input, Error> {
        info!(target: "sdl", "creating event pump");
        let events = sdl.event_pump()?;

        let key_map = match keymap {
            "qwerty" | "QWERTY" => QWERTY_KEY_MAP,
            "colemak" | "COLEMAK" => COLEMAK_KEY_MAP,
            _ => return Err(Error::S("unknown key mapping".into()))
        }.iter()
         .cloned()
         .collect::<HashMap<_, _>>();
        debug!(target: "inp", "key map: {:?}", key_map);

        Ok(Input {
            events,
            key_map,
            key_status: [false; 16],
            last_key: None,
            quit: false,
        })
    }

    pub fn process_pending_input(&mut self) {
        debug!(target: "inp", "processing pending input");
        while let Some(event) = self.events.poll_event() {
            trace!(target: "evt", "processing event {:?}", event);

            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    keymod: Mod::LCTRLMOD,
                    repeat: false,
                    ..
                } |
                Event::Quit { .. } => {
                    self.quit = true;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } if self.key_map.contains_key(&keycode) => {
                    // SAFETY: The value will be present in the map because the
                    //         guard on this match arm guarantees that the key
                    //         is present before matching.
                    let value = *self.key_map.get(&keycode).unwrap();
                    trace!(target: "inp", "processing key down for {:?}", keycode);
                    self.key_down(value);
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } if self.key_map.contains_key(&keycode) => {
                    // SAFETY: The value will be present in the map because the
                    //         guard on this match arm guarantees that the key
                    //         is present before matching.
                    let value = *self.key_map.get(&keycode).unwrap();
                    self.key_up(value);
                }
                _ => {}
            }
        }
    }

    pub fn wait_for_input(&mut self) -> u8 {
        debug!(target: "inp", "waiting for next input");
        self.last_key = None;

        while self.last_key.is_none() {
            self.process_pending_input();
            if self.quit {
                return 0;
            }

            thread::sleep(Duration::from_millis(5));
        }

        // SAFETY: The exit condition of the loop above is that the last_key
        //         field is not None.
        self.last_key.unwrap()
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.key_status[key as usize]
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

pub type KeyMap = [(Keycode, u8)];

#[allow(dead_code)]
pub static QWERTY_KEY_MAP: &KeyMap = &[
    (Keycode::Num1, 0x1),
    (Keycode::Num2, 0x2),
    (Keycode::Num3, 0x3),
    (Keycode::Num4, 0xc),
    (Keycode::Q, 0x4),
    (Keycode::W, 0x5),
    (Keycode::E, 0x6),
    (Keycode::R, 0xd),
    (Keycode::A, 0x7),
    (Keycode::S, 0x8),
    (Keycode::D, 0x9),
    (Keycode::F, 0xe),
    (Keycode::Z, 0xa),
    (Keycode::X, 0x0),
    (Keycode::C, 0xb),
    (Keycode::V, 0xf),
];

#[allow(dead_code)]
pub static COLEMAK_KEY_MAP: &KeyMap = &[
    (Keycode::Num1, 0x1),
    (Keycode::Num2, 0x2),
    (Keycode::Num3, 0x3),
    (Keycode::Num4, 0xc),
    (Keycode::Q, 0x4),
    (Keycode::W, 0x5),
    (Keycode::F, 0x6),
    (Keycode::P, 0xd),
    (Keycode::A, 0x7),
    (Keycode::R, 0x8),
    (Keycode::S, 0x9),
    (Keycode::T, 0xe),
    (Keycode::Z, 0xa),
    (Keycode::X, 0x0),
    (Keycode::C, 0xb),
    (Keycode::V, 0xf),
];
