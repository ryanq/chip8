use log::info;
use sdl2::event::Event;
use sdl2::pixels::Color;
use std::{
    error,
    fmt::{self, Formatter},
    fs::File,
    io::{self, Read},
    thread,
    time::Duration,
};

mod chip8;
mod cli;
mod display;

use chip8::*;
use display::*;

fn main() -> Result<(), Error> {
    let args = cli::process_arguments();
    let log_level = cli::log_level(args.occurrences_of(cli::VERBOSE));
    cli::configure_logging(log_level);

    let program = {
        // SAFETY: The 'program' argument is required, so execution will end
        //         before reaching this block if the program parameter is left
        //         unspecified.
        let path = args.value_of(cli::PROGRAM).unwrap();
        info!("reading program from {}", path);
        let mut file = File::open(path)?;
        let mut buffer = Vec::with_capacity(0x1000);
        file.read_to_end(&mut buffer)?;

        buffer
    };

    let mut display = Display::with_resolution(SCREEN_WIDTH_PIXELS, SCREEN_HEIGHT_PIXELS);

    let mut c8 = Chip8::new();
    c8.load_at(PROGRAM_START, program);

    let scale = match (
        args.occurrences_of(cli::SMALL),
        args.occurrences_of(cli::LARGE),
    ) {
        (0, 0) => 8,
        (_, 0) => 4,
        (0, _) => 16,
        _ => unsafe { std::hint::unreachable_unchecked() },
    };

    let sdl2 = sdl2::init()?;
    let video = sdl2.video()?;
    let window = video
        .window(
            "CHIP-8",
            SCREEN_WIDTH_PIXELS as u32 * scale,
            SCREEN_HEIGHT_PIXELS as u32 * scale,
        )
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut events = sdl2.event_pump()?;
    'exe: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'exe,
                _ => {}
            }
        }

        c8.step(&mut display);

        display.update_screen(&mut canvas, scale)?;
        canvas.present();

        thread::sleep(Duration::from_millis(1000u64 / 60));
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    S(String),
    Sdl(sdl2::IntegerOrSdlError),
    Win(sdl2::video::WindowBuildError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => write!(f, "I/O error: {}", e),
            Error::S(s) => write!(f, "error: {}", s),
            Error::Sdl(e) => write!(f, "SDL error: {}", e),
            Error::Win(e) => write!(f, "error building a window: {}", e),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        Error::IO(inner)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::S(s)
    }
}

impl From<sdl2::video::WindowBuildError> for Error {
    fn from(inner: sdl2::video::WindowBuildError) -> Error {
        Error::Win(inner)
    }
}

impl From<sdl2::IntegerOrSdlError> for Error {
    fn from(inner: sdl2::IntegerOrSdlError) -> Error {
        Error::Sdl(inner)
    }
}
