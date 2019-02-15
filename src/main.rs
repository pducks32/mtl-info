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
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    let input_file_path = matches.value_of("INPUT").unwrap();
    let mut file = File::open(input_file_path)?;

    file.seek(SeekFrom::Start(0x18))?;
    let number_of_entries_offset = file.read_u32::<LittleEndian>()?;
    file.seek(SeekFrom::Start(number_of_entries_offset as u64))?;
    let number_of_entries = file.read_u32::<LittleEndian>()?;
    println!("Number of entries is {}", number_of_entries);

    Ok(())
}
