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

struct HeaderInformation {
    pub entry_headers_offset: u64,
    pub entry_bodys_offset: u64,
    pub number_of_entries: u32,
}

impl HeaderInformation {
    pub fn from_reader<RAndS: Read + Seek>(
        reader: &mut RAndS,
    ) -> Result<HeaderInformation, io::Error> {
        reader.seek(SeekFrom::Start(0x18))?;
        let number_of_entries_offset = reader.read_u32::<LittleEndian>()? as u64;

        reader.seek(SeekFrom::Start(0x48))?;
        let payload_offset = reader.read_u32::<LittleEndian>()? as u64;

        reader.seek(SeekFrom::Start(number_of_entries_offset))?;
        let number_of_entries = reader.read_u32::<LittleEndian>()?;

        return Ok(HeaderInformation {
            entry_headers_offset: number_of_entries_offset + 4,
            entry_bodys_offset: payload_offset,
            number_of_entries,
        });
    }
}

struct MetalLibraryEntry {
    pub name: String,
    pub body_size: u64,
}

struct MetalLibrary {
    header: HeaderInformation,
    entry_stubs: Vec<MetalLibraryEntry>,
}

struct MetalEntryHeaderIterator<'a, T: Read + Seek> {
    reader: &'a mut T,
}

impl<'a, T> Iterator for MetalEntryHeaderIterator<'a, T>
where
    T: Read + Seek,
{
    type Item = HeaderInformation;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl MetalLibrary {
    fn create(header: HeaderInformation, entries: Option<Vec<MetalLibraryEntry>>) -> Self {
        let entry_stubs = entries.unwrap_or_else(|| {
            let mut stubs: Vec<MetalLibraryEntry> = Vec::new();
            stubs.reserve(header.number_of_entries as usize);
            stubs
        });
        MetalLibrary {
            header,
            entry_stubs,
        }
    }
}

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
            Arg::with_name("entries")
                .short("l")
                .long("list-entries")
                .requires("INPUT")
                .conflicts_with("count")
                .help("Returns the entries' names and offsets"),
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

    let header = HeaderInformation::from_reader(&mut file)?;

    if matches.is_present("count") {
        println!("Number of entries is {}", header.number_of_entries);
        return Ok(());
    }

    let mut metal_library = MetalLibrary::create(header, None);

    for _ in 0..metal_library.header.number_of_entries {
        let mut file_name: Option<String> = None;
        let mut body_size = 064;
        file.seek(SeekFrom::Current(4))?; // Entry size is not needed;
        loop {
            let mut tag_type = [0u8; 4];
            file.read(&mut tag_type).unwrap();
            debug!("Tag name {}", std::str::from_utf8(&tag_type).unwrap());
            if tag_type.as_ref() == b"ENDT" {
                trace!("Hit end");
                metal_library.entry_stubs.push(MetalLibraryEntry {
                    name: file_name.unwrap(),
                    body_size,
                });
                break;
            }

            let tag_length = file.read_u16::<LittleEndian>()? as usize;
            match tag_type.as_ref() {
                b"NAME" => {
                    let mut name_buffer = vec![0u8; tag_length];

                    debug!("Size of name {}", tag_length);
                    file.read_exact(&mut name_buffer)?;
                    let name_slice = String::from_utf8(name_buffer).unwrap();
                    info!("Found entry named {}", name_slice);
                    file_name = Some(name_slice);
                }
                b"MDSZ" => {
                    debug!("MDSZ tag length {}", tag_length);
                    let size = file.read_u64::<LittleEndian>()?;
                    info!("\t- Function size {}", size);
                    body_size = size;
                }
                _ => {
                    debug!("No match for tag length {}", tag_length);
                    file.seek(SeekFrom::Current(tag_length as i64))?;
                }
            }
        }
    }

    Ok(())
}
