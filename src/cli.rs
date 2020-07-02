use {
    clap::{crate_authors, App, Arg, ArgMatches},
    log::LevelFilter,
    std::io::Write,
};

pub const VERBOSE: &'static str = "verbose";
pub const SMALL: &'static str = "small";
pub const LARGE: &'static str = "large";
pub const KEYMAP: &'static str = "keymap";
pub const PROGRAM: &'static str = "program";

pub fn process_arguments<'a>() -> ArgMatches<'a> {
    App::new("chip8")
        .version("1.0")
        .author(crate_authors!())
        .about("A Chip-8 VM that implements the original standard")
        .arg(
            Arg::with_name(VERBOSE)
                .short("v")
                .long("verbose")
                .takes_value(false)
                .multiple(true)
                .help("Prints verbose output"),
        )
        .arg(
            Arg::with_name(SMALL)
                .short("s")
                .long("small")
                .takes_value(false)
                .overrides_with(LARGE)
                .help("Render the UI smaller"),
        )
        .arg(
            Arg::with_name(LARGE)
                .short("l")
                .long("large")
                .takes_value(false)
                .help("Render the UI larger"),
        )
        .arg(
            Arg::with_name(KEYMAP)
                .short("k")
                .long("keymap")
                .takes_value(true)
                .possible_values(&["qwerty", "colemak"])
                .default_value("qwerty")
                .help("Switch key mapping"),
        )
        .arg(
            Arg::with_name(PROGRAM)
                .value_name("PROGRAM")
                .required(true)
                .help("The path to a Chip-8 program binary"),
        )
        .get_matches()
}

pub fn log_level(count: u64) -> LevelFilter {
    match count {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

pub fn configure_logging(log_level: LevelFilter) {
    env_logger::builder()
        .format(|f, record| writeln!(f, "{:>5}: {}", record.level(), record.args()))
        .filter_level(log_level)
        .init();
}
