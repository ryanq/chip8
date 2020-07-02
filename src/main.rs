mod audio;
mod chip8;
mod cli;
mod display;
mod input;

use {
    chip8::Chip8,
    clap::Clap,
    cli::Config,
    std::{error, fmt::{self, Formatter}, io},
};

fn main() -> Result<(), Error> {
    cli::configure_logging();

    let config = Config::parse();

    let mut c8 = Chip8::new(&config)?;
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
