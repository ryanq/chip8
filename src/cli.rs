use {
    clap::Clap,
    log::LevelFilter,
    std::{io::Write, path::PathBuf},
};

#[derive(Clap, Debug)]
#[clap(author,about,version)]
pub struct Config {
    /// Sets the key mapping to use
    #[clap(short, long, arg_enum, env = "CHIRP_KEYMAP", default_value = "qwerty")]
    pub keymap: Keymap,
    /// Sets the rendering size
    #[clap(short, long, arg_enum, default_value = "normal")]
    pub size: Size,
    /// Sets Logging level
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Path to a Chip-8 binary
    pub program: PathBuf,
}

#[derive(Clap, Debug)]
pub enum Keymap {
    Colemak,
    Qwerty,
}

#[derive(Clap, Debug)]
pub enum Size {
    Small,
    Normal,
    Large,
}

pub fn configure_logging(level: u8) {
    env_logger::builder()
        .format(|f, record| writeln!(f, "{:>5}: {}", record.level(), record.args()))
        .filter_level(match level {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();
}
