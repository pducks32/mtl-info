extern crate byteorder;
extern crate log;
use log::{debug, info, trace};

use byteorder::{LittleEndian, ReadBytesExt};
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

pub(crate) struct MetalLibraryEntry {
  pub name: String,
  pub body_size: u64,
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

pub(crate) struct Tag {
  code: [u8; 4],
  length: u16,
}

pub(crate) enum EntryHeaderTag {
  Name(String),
  Size(u64),
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
    debug!("Tag name {}", std::str::from_utf8(&tag_type).unwrap());
    if tag_type.as_ref() == b"ENDT" {
      trace!("Hit end");
      return Some(EntryHeaderTag::End);
    }

    let tag_length = self.reader.read_u16::<LittleEndian>().unwrap() as usize;
    match tag_type.as_ref() {
      b"NAME" => {
        let mut name_buffer = vec![0u8; tag_length];

        debug!("Size of name {}", tag_length);
        self.reader.read_exact(&mut name_buffer).unwrap();
        let name_slice = String::from_utf8(name_buffer).unwrap();
        info!("Found entry named {}", name_slice);
        Some(EntryHeaderTag::Name(name_slice))
      }
      b"MDSZ" => {
        debug!("MDSZ tag length {}", tag_length);
        let size = self.reader.read_u64::<LittleEndian>().unwrap();
        info!("\t- Function size {}", size);
        Some(EntryHeaderTag::Size(size))
      }
      _ => {
        debug!("No match for tag length {}", tag_length);
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
    return (bound.unwrap_or_default(), bound);
  }

  fn next(&mut self) -> Option<Self::Item> {
    let mut file_name: Option<String> = None;
    let mut body_size = 064;
    self.reader.seek(SeekFrom::Current(4)).unwrap(); // Entry size is not needed;
    let iterator = EntryTagIterator {
      reader: self.reader,
    };

    for tag in iterator {
      match tag {
        EntryHeaderTag::Name(name) => file_name = Some(name),
        EntryHeaderTag::Size(size) => body_size = size,
        EntryHeaderTag::End => break,
        _ => continue,
      }
    }
    self.number_of_items_read += 1;
    file_name.map(|name| MetalLibraryEntry { name, body_size })
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
