use std::fmt;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

use xmas_elf::program::Type as ProgramType;
use xmas_elf::sections::ShType;
use xmas_elf::ElfFile;

pub struct ProgramDescription {
    /// Virtual address of .text section in RAM
    pub text_offset: u32,

    /// Size of the .text section in RAM
    pub text_size: u32,

    /// Virtual address of .data section in RAM
    pub data_offset: u32,

    /// Size of .data section
    pub data_size: u32,

    /// Size of the .bss section
    pub bss_size: u32,

    /// Virtual address of the entrypoint
    pub entry_point: u32,

    /// Program contents
    pub program: Vec<u8>,
}

#[derive(Debug)]
pub enum ElfReadError {
    /// Couldn't read ELF file
    ReadFileError(std::io::Error),

    /// Couldn't open the ELF file
    OpenElfError(std::io::Error),

    /// Couldn't parse the ELF file
    ParseElfError(&'static str),

    /// Section wasn't word-aligned
    SectionNotAligned(String /* section name */, usize /* section size */),

    /// Couldn't seek the file to write the section
    FileSeekError(std::io::Error),

    /// Couldn't write the section to the file
    WriteSectionError(std::io::Error),
}

impl fmt::Display for ElfReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ElfReadError::*;
        match self {
            ReadFileError(e) => write!(f, "couldn't read from the file: {}", e),
            OpenElfError(e) => write!(f, "couldn't open the elf file: {}", e),
            ParseElfError(e) => write!(f, "couldn't parse the elf file: {}", e),
            SectionNotAligned(s, a) => write!(f, "elf section {} had unaligned length {}", s, a),
            FileSeekError(e) => write!(f, "couldn't seek in the output file: {}", e),
            WriteSectionError(e) => write!(f, "couldn't write a section to the output file: {}", e),
        }
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn read_program<P: AsRef<Path>>(filename: P) -> Result<ProgramDescription, ElfReadError> {
    let mut b = Vec::new();
    {
        let mut fi = File::open(filename).map_err(ElfReadError::OpenElfError)?;
        fi.read_to_end(&mut b).map_err(ElfReadError::ReadFileError)?;
    }
    process_program(&b)
}

pub fn process_program(b: &[u8]) -> Result<ProgramDescription, ElfReadError> {
    let elf = ElfFile::new(b).map_err(ElfReadError::ParseElfError)?;
    let entry_point = elf.header.pt2.entry_point() as u32;
    let mut program_data = Cursor::new(Vec::new());

    let mut size = 0;
    let mut data_offset = 0;
    let mut data_size = 0;
    let mut text_offset = 0;
    let mut text_size = 0;
    let mut bss_size = 0;
    let mut phys_offset = 0;

    for ph in elf.program_iter() {
        if ph.get_type() == Ok(ProgramType::Load) && phys_offset == 0 {
            phys_offset = ph.physical_addr();
        }
    }

    let mut program_offset = 0;
    for s in elf.section_iter() {
        let name = s.get_name(&elf).unwrap_or("<<error>>");

        if s.address() == 0 {
            continue;
        }

        size += s.size();
        // Pad the section so it's a multiple of 4 bytes.
        // It's unclear if this is necessary, since this seems to indicate
        // that something has gone horribly wrong.
        size += (4 - (size & 3)) & 3;
        if size & 3 != 0 {
            return Err(ElfReadError::SectionNotAligned(name.to_owned(), s.size() as usize));
        }

        if name == ".data" {
            data_offset = s.address() as u32;
            data_size += s.size() as u32;
        } else if s.get_type() == Ok(ShType::NoBits) {
            // Add bss-type sections to the data section
            bss_size += s.size() as u32;
            continue;
        } else if text_offset == 0 && (s.address() != 0 || s.size() != 0) {
            text_offset = s.address() as u32;
            text_size += s.size() as u32;
        } else {
            if text_offset + text_size != s.address() as u32 {
                let bytes_to_add = s.address() - (text_offset + text_size) as u64;
                program_data
                    .seek(SeekFrom::Current(bytes_to_add as i64))
                    .map_err(ElfReadError::FileSeekError)?;
                text_size += bytes_to_add as u32;
                program_offset += bytes_to_add;
                // panic!(
                //     "size not correct!  should be {:08x}, was {:08x}, need to add {} bytes",
                //     text_offset + text_size,
                //     s.address(),
                //     s.address() - (text_offset + text_size) as u64,
                // );
            }
            text_size += s.size() as u32;
        }
        if s.size() == 0 {
            continue;
        }
        let section_data = s.raw_data(&elf);
        program_data.seek(SeekFrom::Start(program_offset)).map_err(ElfReadError::FileSeekError)?;
        program_data.write(section_data).map_err(ElfReadError::WriteSectionError)?;
        program_offset += section_data.len() as u64;
    }

    Ok(ProgramDescription {
        entry_point,
        program: program_data.into_inner(),
        data_size,
        data_offset,
        text_offset,
        text_size,
        bss_size,
    })
}
