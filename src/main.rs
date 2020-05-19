mod chip8;
mod cli;
mod display;
mod input;

use {
    chip8::Chip8,
    log::info,
    std::{
        error,
        fmt::{self, Formatter},
        fs::File,
        io::{self, Read},
    },
};

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

    let gui_scale = if args.occurrences_of(cli::SMALL) != 0 {
        4
    } else if args.occurrences_of(cli::LARGE) != 0 {
        16
    } else {
        8
    };

    let mut c8 = Chip8::new(&program, gui_scale)?;
    c8.run()?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
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
