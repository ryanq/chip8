use clap::{crate_authors, App, Arg};
use log::LevelFilter;
use minifb::{Window, WindowOptions};
use std::{
    error,
    fmt::{self, Formatter},
    fs::File,
    io::{self, Read, Write},
    time::Duration,
};

mod chip8;
mod display;

use chip8::*;
use display::*;

fn main() -> Result<(), Error> {
    let args = App::new("chip8")
        .version("1.0")
        .author(crate_authors!())
        .about("A Chip-8 VM that implements the original standard")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .multiple(true)
                .help("Prints verbose output"),
        )
        .arg(
            Arg::with_name("small")
                .short("s")
                .long("small")
                .takes_value(false)
                .overrides_with("large")
                .help("Render the UI smaller"),
        )
        .arg(
            Arg::with_name("large")
                .short("l")
                .long("large")
                .takes_value(false)
                .help("Render the UI larger"),
        )
        .arg(
            Arg::with_name("program")
                .value_name("PROGRAM")
                .required(true)
                .help("The path to a Chip-8 program binary"),
        )
        .get_matches();

    let mut logger = env_logger::Builder::from_default_env();
    logger
        .format(|f, record| {
            writeln!(
                f,
                "{:5}({}): {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter_level(match args.occurrences_of("verbose") {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();

    let program = {
        // SAFETY: The 'program' argument is required, so execution will end
        //         before reaching this block.
        let path = args.value_of("program").unwrap();
        let mut file = File::open(path)?;
        let mut buffer = Vec::with_capacity(0x1000);
        file.read_to_end(&mut buffer)?;

        buffer
    };

    let mut display = Display::with_resolution(SCREEN_WIDTH_PIXELS, SCREEN_HEIGHT_PIXELS);

    let mut c8 = Chip8::new();
    c8.load_at(PROGRAM_START, program);

    let scale = if args.occurrences_of("small") > 0 {
        4
    } else if args.occurrences_of("large") > 0 {
        16
    } else {
        8
    };
    let mut window = Window::new(
        "CHIP-8",
        SCREEN_WIDTH_PIXELS * scale,
        SCREEN_HEIGHT_PIXELS * scale,
        WindowOptions::default(),
    )?;
    window.limit_update_rate(Some(Duration::from_micros(16600)));

    while window.is_open() {
        c8.step(&mut display);
        window.update_with_buffer(display.buffer(), SCREEN_WIDTH_PIXELS, SCREEN_HEIGHT_PIXELS)?;
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    UI(minifb::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => write!(f, "I/O error: {}", e),
            Error::UI(e) => write!(f, "error: {}", e),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        Error::IO(inner)
    }
}

impl From<minifb::Error> for Error {
    fn from(inner: minifb::Error) -> Error {
        Error::UI(inner)
    }
}
