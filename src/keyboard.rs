use {
    crate::Error,
    sdl2::{event::Event, EventPump, Sdl},
};

#[derive(PartialEq)]
pub enum Action {
    Continue,
    Quit,
}

pub struct Keyboard {
    events: EventPump,
}

impl Keyboard {
    pub fn new(sdl: &Sdl) -> Result<Keyboard, Error> {
        let events = sdl.event_pump()?;

        Ok(Keyboard { events })
    }

    pub fn handle_input(&mut self) -> Action {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. } => return Action::Quit,
                _ => (),
            }
        }

        Action::Continue
    }
}
