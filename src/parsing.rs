extern crate byteorder;
extern crate colored;
extern crate log;
use colored::*;
use log::{debug, info, log_enabled, trace, Level};

use std::convert::TryInto;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

pub(crate) struct HeaderInformation {
  pub entry_headers_offset: u64,
  pub entry_bodys_offset: u64,
  pub number_of_entries: u32,
}

impl HeaderInformation {
  pub fn from_reader<RAndS: Read + Seek>(
    reader: &mut RAndS,
  ) -> Result<HeaderInformation, io::Error> {
    reader.seek(SeekFrom::Start(0x18))?;
    let number_of_entries_offset = u64::from(reader.read_u32::<LittleEndian>()?);

    reader.seek(SeekFrom::Start(0x48))?;
    let payload_offset = u64::from(reader.read_u32::<LittleEndian>()?);

    reader.seek(SeekFrom::Start(number_of_entries_offset))?;
    let number_of_entries = reader.read_u32::<LittleEndian>()?;

    Ok(HeaderInformation {
      entry_headers_offset: number_of_entries_offset + 4,
      entry_bodys_offset: payload_offset,
      number_of_entries,
    })
  }
}

pub(crate) struct MetalLibraryEntry {
  pub name: String,
  pub body_size: u64,
  pub body_offset: u64,
}

pub(crate) struct MetalLibrary {
  pub header: HeaderInformation,
  pub entry_stubs: Vec<MetalLibraryEntry>,
}

pub(crate) struct MetalEntryHeaderIterator<'a, T: Read + Seek> {
  pub reader: &'a mut T,
  pub number_of_items: Option<usize>,
  pub number_of_items_read: usize,
}

#[allow(dead_code)]
pub(crate) struct Tag {
  code: [u8; 4],
  length: u16,
}

pub(crate) enum EntryHeaderTag {
  Name(String),
  Size(u64),
  Offset(u64),
  End,
  Other(Tag),
}

pub(crate) struct EntryTagIterator<'a, T: Read + Seek> {
  reader: &'a mut T,
}

impl<'a, T> Iterator for EntryTagIterator<'a, T>
where
  T: Read + Seek,
{
  type Item = EntryHeaderTag;

  fn next(&mut self) -> Option<Self::Item> {
    let mut tag_type = [0u8; 4];
    self.reader.read(&mut tag_type).ok();
    debug!(
      "Tag name {}",
      std::str::from_utf8(&tag_type).unwrap().bold()
    );
    if tag_type.as_ref() == b"ENDT" {
      trace!("-------------");
      return Some(EntryHeaderTag::End);
    }

    let tag_length = self.reader.read_u16::<LittleEndian>().unwrap() as usize;
    debug!("\t- Tag Length {}", format!("{}", tag_length).bold());
    match tag_type.as_ref() {
      b"NAME" => {
        let mut name_buffer = vec![0u8; tag_length];

        self.reader.read_exact(&mut name_buffer).unwrap();
        let mut name_slice = String::from_utf8(name_buffer).unwrap();
        name_slice.pop(); // Remove \0
        info!("\t- Name: {}", format!("{:?}", name_slice).bold());
        Some(EntryHeaderTag::Name(name_slice))
      }
      b"MDSZ" => {
        let size = self.reader.read_u64::<LittleEndian>().unwrap();
        info!("\t- Function size {}", format!("{}", size).bold());
        Some(EntryHeaderTag::Size(size))
      }
      b"OFFT" => {
        let mut name_buffer = vec![0u8; tag_length];
        self.reader.read_exact(&mut name_buffer).unwrap();

        if log_enabled!(Level::Trace) {
          name_buffer.chunks_exact(8).for_each(|chunk| {
            let value = u64::from_le_bytes(chunk.try_into().unwrap());
            trace!("\t- Value: {}", format!("{:?}", value).bold());
          });
        }

        let offset = u64::from_le_bytes(name_buffer[16..24].try_into().unwrap());
        Some(EntryHeaderTag::Offset(offset))
      }
      _ => {
        debug!("\t-NO MATCH");
        self
          .reader
          .seek(SeekFrom::Current(tag_length as i64))
          .unwrap();
        Some(EntryHeaderTag::Other(Tag {
          code: tag_type,
          length: tag_length as u16,
        }))
      }
    }
  }
}

impl<'a, T> Iterator for MetalEntryHeaderIterator<'a, T>
where
  T: Read + Seek,
{
  type Item = MetalLibraryEntry;

  fn size_hint(&self) -> (usize, Option<usize>) {
    let bound = self.number_of_items.map(|m| m - self.number_of_items_read);
    (bound.unwrap_or_default(), bound)
  }

  fn next(&mut self) -> Option<Self::Item> {
    let mut file_name: Option<String> = None;
    let mut body_size = 0u64;
    let mut body_offset = 0u64;
    self.reader.seek(SeekFrom::Current(4)).unwrap(); // Entry size is not needed;
    let iterator = EntryTagIterator {
      reader: self.reader,
    };

    for tag in iterator {
      match tag {
        EntryHeaderTag::Name(name) => file_name = Some(name),
        EntryHeaderTag::Size(size) => body_size = size,
        EntryHeaderTag::Offset(offset) => body_offset = offset,
        EntryHeaderTag::End => break,
        _ => continue,
      }
    }
    self.number_of_items_read += 1;
    file_name.map(|name| MetalLibraryEntry {
      name,
      body_size,
      body_offset,
    })
  }
}

impl MetalLibrary {
  pub fn create(header: HeaderInformation, entries: Option<Vec<MetalLibraryEntry>>) -> Self {
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

/// Creates a metal library parser around a `Read` and `Seek`
/// data set.
///
/// It maintains an internal `state: ParsingState` to avoid
/// redundant processing. Most uses of `Parser<T>` should
/// therefore be mutable.
pub struct Parser<'a, T: Read + Seek> {
  reader: &'a mut T,
  state: ParsingState,
}

/// State of a `Parser`
///
/// Parsers will transition from `Initial`
/// to `Header` once the library's header has been
/// read. Then it will be ready to read the entry
/// headers which will cause a transiton to
/// `EntryStubs`
enum ParsingState {
  /// No processing has yet happened.
  Initial,
  /// Library header has been read.
  Header(HeaderInformation),
  /// Entry headers have been read.
  EntryStubs(MetalLibrary),
}

impl<'a> Parser<'a, File> {
  /// Create a `Parser` object around a file.
  ///
  /// File must be mutable since it must be read.
  pub fn with_file(file: &'_ mut File) -> Parser<'_, File> {
    Parser {
      reader: file,
      state: ParsingState::Initial,
    }
  }
}

impl<'a, T> Parser<'a, T>
where
  T: Read + Seek,
{
  /// Checks that a reader can be understood as a metallib file
  ///
  /// # Remarks
  ///
  /// Apple is fond of 4 byte header codes. Metal Libraries
  /// use the `"MTLB"` designation which should exist at
  /// mark 0.
  pub fn is_metal_library_file(reader: &mut T) -> bool {
    let mut magic_bytes = [0u8; 4];
    reader
      .seek(SeekFrom::Start(0))
      .expect("to be able to seek to beginning");
    reader
      .read_exact(&mut magic_bytes)
      .expect("to be able to read 4 bytes");
    magic_bytes.as_ref() == b"MTLB"
  }

  pub(crate) fn header(&mut self) -> &HeaderInformation {
    match self.state {
      ParsingState::Header(ref header) => header,
      ParsingState::EntryStubs(ref lib) => &lib.header,
      ParsingState::Initial => {
        let header = HeaderInformation::from_reader(self.reader).unwrap();
        self.state = ParsingState::Header(header);
        match self.state {
          ParsingState::Header(ref h) => h,
          _ => unreachable!(),
        }
      }
    }
  }

  pub(crate) fn library(&mut self) -> &MetalLibrary {
    let _ensure_header = self.header();
    let old_state = std::mem::replace(&mut self.state, ParsingState::Initial);
    self.state = Parser::eat_state(old_state, self.reader);
    match self.state {
      ParsingState::EntryStubs(ref lib) => lib,
      _ => unreachable!(),
    }
  }

  pub(crate) fn read_from_offset(&mut self, offset: u64, mut buffer: &mut [u8]) {
    self
      .reader
      .seek(SeekFrom::Start(offset))
      .expect("To be able to seek");
    self
      .reader
      .read_exact(&mut buffer)
      .expect("to be able to read");
  }

  fn eat_state(state: ParsingState, reader: &mut T) -> ParsingState {
    let header = match state {
      ParsingState::Header(h) => h,
      _ => unreachable!(),
    };
    let iterator = MetalEntryHeaderIterator {
      reader,
      number_of_items: Some(header.number_of_entries as usize),
      number_of_items_read: 0,
    };
    let entries = iterator.take(header.number_of_entries as usize);

    let metal_library = MetalLibrary::create(header, Some(entries.collect()));
    ParsingState::EntryStubs(metal_library)
  }
}
