use clap::{crate_authors, App, Arg};
use log::LevelFilter;
use std::{
    fs::File,
    io::{self, Read, Write},
};

mod chip8;
use chip8::*;

fn main() -> Result<(), io::Error> {
    let args = App::new("chip8")
        .version("1.0")
        .author(crate_authors!())
        .about("A Chip-8 VM that implements the original standard")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .help("Prints verbose output"),
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
            0 => LevelFilter::Warn,
            _ => LevelFilter::Debug,
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

    let mut c8 = Chip8::new();
    c8.load_at(PROGRAM_START, program);
    c8.run();

    Ok(())
}
