//! Facilities for writing an Xcursor to disk, which isn't
//! currently supported by `xcursor`.

// Information about Xcursor files can be found here:
// https://www.x.org/archive/X11R7.7/doc/man/man3/Xcursor.3.xhtml

use byteorder::{LittleEndian, WriteBytesExt};
use std::{io::Write, mem::size_of};

const XCURSOR_MAGIC: &[u8] = b"Xcur";
const SIZE_OF_U32: u32 = size_of::<u32>() as u32;
// Four fields: `magic`, `header[_size]`, `version`, and `ntoc`.
const HEADER_BYTE_LENGTH: u32 = SIZE_OF_U32 * 4;

#[derive(Clone, Copy)]
pub(crate) struct TocEntry {
    entry_type: u32,
    subtype: u32,
    position: u32,
}

impl TocEntry {
    const BYTE_LENGTH: u32 = SIZE_OF_U32 * 3;

    fn write_to(&self, mut writer: impl Write) -> anyhow::Result<()> {
        writer.write_u32::<LittleEndian>(self.entry_type)?;
        writer.write_u32::<LittleEndian>(self.subtype)?;
        writer.write_u32::<LittleEndian>(self.position)?;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct Image {
    // This is the `subtype` field
    size: u32,
    width: u32,
    height: u32,
    xhot: u32,
    yhot: u32,
    delay: u32,
    pixels: Vec<u32>,
}

impl Image {
    const TYPE: u32 = 0xFFFD0002;
    const HEADER_SIZE: u32 = 36;
    const VERSION: u32 = 1;

    pub(crate) fn new(
        size: u32,
        width: u32,
        height: u32,
        xhot: u32,
        yhot: u32,
        delay: u32,
        pixels: Vec<u32>,
    ) -> anyhow::Result<Self> {
        if width * height != pixels.len().try_into()? {
            panic!(
                "image dimensions ({}x{} = {}) do not match the number of pixels given ({})",
                width,
                height,
                width * height,
                pixels.len()
            );
        }

        Ok(Self {
            size,
            width,
            height,
            xhot,
            yhot,
            delay,
            pixels,
        })
    }

    fn write_to(&self, mut writer: impl Write) -> anyhow::Result<()> {
        writer.write_u32::<LittleEndian>(Self::HEADER_SIZE)?;
        writer.write_u32::<LittleEndian>(Self::TYPE)?;
        // `subtype`
        writer.write_u32::<LittleEndian>(self.size)?;
        writer.write_u32::<LittleEndian>(Self::VERSION)?;
        writer.write_u32::<LittleEndian>(self.width)?;
        writer.write_u32::<LittleEndian>(self.height)?;
        writer.write_u32::<LittleEndian>(self.xhot)?;
        writer.write_u32::<LittleEndian>(self.yhot)?;
        writer.write_u32::<LittleEndian>(self.delay)?;

        for pixel in &self.pixels {
            writer.write_u32::<LittleEndian>(*pixel)?;
        }

        Ok(())
    }

    fn byte_length(&self) -> anyhow::Result<u32> {
        // Images have nine `u32` fields, including `header`, `type`, and `version`.
        Ok(SIZE_OF_U32 * 9 + u32::try_from(self.pixels.len())? * SIZE_OF_U32)
    }
}

#[derive(Clone)]
pub(crate) struct Xcursor {
    table_of_contents: Vec<TocEntry>,
    chunks: Vec<Image>,
    /// The next `TocEntry::position` for a given chunk.
    next_chunk_position: u32,
}

impl Xcursor {
    pub(crate) fn new() -> Self {
        let mut res = Self {
            table_of_contents: Vec::new(),
            chunks: Vec::new(),
            next_chunk_position: 0,
        };

        // `header_byte_length` shouldn't fail since `res.table_of_contents` is empty.
        // The next chunk position will be shifted right by the new table of contents entry.
        res.next_chunk_position = HEADER_BYTE_LENGTH + TocEntry::BYTE_LENGTH;
        res
    }

    pub(crate) fn add_chunk(&mut self, image: Image) -> anyhow::Result<()> {
        // Unfortunately, to account for this new image's `TocEntry`, we have to update
        // the position in *every existing `TocEntry`*
        for toc_entry in &mut self.table_of_contents {
            toc_entry.position += TocEntry::BYTE_LENGTH;
        }

        self.table_of_contents.push(TocEntry {
            entry_type: Image::TYPE,
            subtype: image.size,
            position: self.next_chunk_position,
        });

        self.next_chunk_position += TocEntry::BYTE_LENGTH + image.byte_length()?;
        self.chunks.push(image);
        Ok(())
    }

    pub(crate) fn write_to(&self, mut writer: impl Write) -> anyhow::Result<()> {
        writer.write_all(XCURSOR_MAGIC)?;
        writer.write_u32::<LittleEndian>(16)?;
        // File version, taken from a random Xcursor (perhaps it's 2 bytes for major, 2 bytes for minor?)
        writer.write_u32::<LittleEndian>(0x00010000)?;
        writer.write_u32::<LittleEndian>(self.table_of_contents.len().try_into()?)?;

        for toc_entry in &self.table_of_contents {
            toc_entry.write_to(&mut writer)?;
        }

        for chunk in &self.chunks {
            chunk.write_to(&mut writer)?;
        }

        Ok(())
    }
}
