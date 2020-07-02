use {
    clap::Clap,
    log::LevelFilter,
    std::{
        io::Write,
        path::PathBuf,
    },
};

/// A Chip-8 interpreter
#[derive(Clap, Debug)]
pub struct Config {
    /// Set the key mapping to use
    #[clap(short, long, arg_enum, default_value = "qwerty")]
    pub keymap: Keymap,
    /// Set the rendering size
    #[clap(short, long, arg_enum, default_value = "normal")]
    pub size: Size,
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

// pub fn process_arguments<'a>() -> ArgMatches<'a> {
//     App::new("chip8")
//         .version("1.0")
//         .author(crate_authors!())
//         .about("A Chip-8 VM that implements the original standard")
//         .arg(
//             Arg::with_name(PROGRAM)
//                 .value_name("PROGRAM")
//                 .required(true)
//                 .help("The path to a Chip-8 program binary"),
//         )
//         .get_matches()
// }

pub fn configure_logging() {
    env_logger::builder()
        .format(|f, record| writeln!(f, "{:>5}: {}", record.level(), record.args()))
        .filter_level(LevelFilter::Error)
        .init();
}
