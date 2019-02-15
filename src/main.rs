extern crate byteorder;
extern crate clap;
extern crate log;
extern crate simple_logger;

use byteorder::{LittleEndian, ReadBytesExt};
use clap::{App, Arg};
use log::{debug, info, trace, Level};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

fn main() -> io::Result<()> {
    let matches = App::new("mtl-info")
        .version("1.0")
        .author("Patrick M. <git@metcalfe.rocks>")
        .about("Read's information from metallib files.")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("count")
                .short("n")
                .long("count")
                .requires("INPUT")
                .help("Return number of functions found in INPUT"),
        )
        .arg(
            Arg::with_name("verbosity")
                .long("verbosity")
                .takes_value(true)
                .default_value("2")
                .help("Set's the logger level. Between 1 and 4"),
        )
        .get_matches();

    let verbosity = matches
        .value_of("verbosity")
        .unwrap()
        .parse::<u8>()
        .unwrap();

    let level = match verbosity {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        x if x >= 4 => Level::Trace,
        _ => Level::Trace,
    };

    simple_logger::init_with_level(level).unwrap();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    let input_file_path = matches.value_of("INPUT").unwrap();
    let mut file = File::open(input_file_path)?;

    file.seek(SeekFrom::Start(0x18))?;
    let number_of_entries_offset = file.read_u32::<LittleEndian>()? as u64;
    file.seek(SeekFrom::Start(number_of_entries_offset))?;
    let number_of_entries = file.read_u32::<LittleEndian>()?;

    if matches.is_present("count") {
    println!("Number of entries is {}", number_of_entries);
    }

    Ok(())
}
