use {
    crate::Error,
    sdl2::{audio::{AudioCallback, AudioDevice, AudioSpecDesired}, Sdl},
};

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Audio {
    device: AudioDevice<SquareWave>,
}

impl Audio {
    pub fn new(sdl: &Sdl) -> Result<Audio, Error> {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let audio = sdl.audio()?;
        let device = audio.open_playback(None, &desired_spec, |spec| {
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25
            }
        })?;

        Ok(Audio {
            device
        })
    }

    pub fn start(&self) {
        self.device.resume();
    }

    pub fn stop(&self) {
        self.device.pause();
    }
}
