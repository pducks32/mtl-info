extern crate byteorder;
extern crate clap;
extern crate log;
extern crate simple_logger;

use log::Level;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod cli;
mod parsing;

use crate::parsing::Parser;

fn main() -> io::Result<()> {
    let matches = cli::build();
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
    let mut parser = Parser::with_file(&mut file);

    match matches.subcommand_name() {
        Some("count") => {
            println!("Number of entries is {}", parser.header().number_of_entries);
        }
        Some("list") => {
            parser.library().entry_stubs.iter().for_each(|entry| {
                println!("Function Name: {}", entry.name);
            });
        }
        Some("bitcode") => {
            let library = parser.library();
            let first_entry = library.entry_stubs.first().expect("First entry");
            let start = library.header.entry_bodys_offset;
            let mut body_buffer = vec![0u8; first_entry.body_size as usize];
            parser.read_from_offset(start, &mut body_buffer);
            std::io::stdout()
                .lock()
                .write_all(&body_buffer)
                .expect("to be able to write out data");
        }
        Some(_) => unreachable!(),
        None => unreachable!(),
    };

    Ok(())
}
