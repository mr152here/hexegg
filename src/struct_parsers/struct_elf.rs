use crate::signatures::is_signature;
use crate::location_list::{Location, LocationList};
use crate::struct_parsers::*;

struct ProgramHeader {
    p_offset: usize,
    v_offset: usize,
    v_size: usize
}

pub fn parse_elf_struct(data: &[u8]) -> Result<LocationList, String> {

    if !is_signature(data, "elf") {
        return Err("Invalid 'ELF' signature!".to_owned());

    } else if data.len() < 52 {
        return Err("Too small for 'ELF' header!".to_owned());
    }

    // ELF header
    let mut header = LocationList::new();
    header.add_location(Location {name: "-- ELF --".to_owned(), offset: 0, size: 0});
    header.add_location(Location {name: "magic".to_owned(), offset: 0, size: 4});
    header.add_location(Location {name: "class".to_owned(), offset: 4, size: 1});
    header.add_location(Location {name: "endianness".to_owned(), offset: 5, size: 1});
    header.add_location(Location {name: "version".to_owned(), offset: 6, size: 1});
    header.add_location(Location {name: "OS_ABI".to_owned(), offset: 7, size: 1});
    header.add_location(Location {name: "ABI_version".to_owned(), offset: 8, size: 1});
    header.add_location(Location {name: "reserved".to_owned(), offset: 9, size: 7});
    header.add_location(Location {name: "type".to_owned(), offset: 16, size: 2});
    header.add_location(Location {name: "machine".to_owned(), offset: 18, size: 2});
    header.add_location(Location {name: "version".to_owned(), offset: 20, size: 4});

    let elf32 = match read_u8(data, 4) {
        Some(b) if b == 1 => true,
        Some(b) if b == 2 => false,
        Some(b) => return Err(format!("Invalid EI_CLASS '{b}'. Is neither 32 nor 64 bit!")),
        None => return Err("ELF header is truncated!".to_owned()),
    };

    let little_endian = match read_u8(data, 5) {
        Some(b) if b == 1 => true,
        Some(b) if b == 2 => false,
        Some(b) => return Err(format!("Invalid EI_DATA '{b}'. Is neither little nor big endian!")),
        None => return Err("ELF header is truncated!".to_owned()),
    };

    //set read data function acording to little/big endianness
    let read_u16 = if little_endian { read_le_u16 } else { read_be_u16 };
    let read_u32 = if little_endian { read_le_u32 } else { read_be_u32 };
    let read_u64 = if little_endian { read_le_u64 } else { read_be_u64 };

    let (mut last_offset, ph_offset, sh_offset, entry_point) = if elf32 {
        header[0].name = "-- ELF32 --".to_owned();
        header.add_location(Location {name: "entry_va".to_owned(), offset: 24, size: 4});
        header.add_location(Location {name: "entry_point".to_owned(), offset: 0, size: 0});
        header.add_location(Location {name: "ph_offset".to_owned(), offset: 28, size: 4});
        header.add_location(Location {name: "sh_offset".to_owned(), offset: 32, size: 4});

        let entry_point = match read_u32(data, 24) {
            Some(v) => v as usize,
            None => return Err("ELF header is truncated!".to_owned()),
        };

        let ph_offset = match read_u32(data, 28) {
            Some(v) => v as usize,
            None => return Err("ELF header is truncated!".to_owned()),
        };

        let sh_offset = match read_u32(data, 32) {
            Some(v) => v as usize,
            None => return Err("ELF header is truncated!".to_owned()),
        };

        (36, ph_offset, sh_offset, entry_point)

    } else {
        header[0].name = "-- ELF64 --".to_owned();
        header.add_location(Location {name: "entry_va".to_owned(), offset: 24, size: 8});
        header.add_location(Location {name: "entry_point".to_owned(), offset: 0, size: 0});
        header.add_location(Location {name: "ph_offset".to_owned(), offset: 32, size: 8});
        header.add_location(Location {name: "sh_offset".to_owned(), offset: 40, size: 8});

        let entry_point = match read_u64(data, 24) {
            Some(v) => v as usize,
            None => return Err("ELF header is truncated!".to_owned()),
        };

        let ph_offset = match read_u64(data, 32) {
            Some(v) => v as usize,
            None => return Err("ELF header is truncated!".to_owned()),
        };

        let sh_offset = match read_u64(data, 40) {
            Some(v) => v as usize,
            None => return Err("ELF header is truncated!".to_owned()),
        };

        (48, ph_offset, sh_offset, entry_point)
    };

    //TODO: there are some complication in specification when ph_num == 0xffff; sh_num >= 0xff00; sh_str_index >= 0xff00
    header.add_location(Location {name: "flags".to_owned(), offset: last_offset, size: 4});
    header.add_location(Location {name: "eh_size".to_owned(), offset: last_offset+4, size: 2});
    header.add_location(Location {name: "ph_entry_size".to_owned(), offset: last_offset+6, size: 2});
    header.add_location(Location {name: "ph_num".to_owned(), offset: last_offset+8, size: 2});
    header.add_location(Location {name: "sh_entry_size".to_owned(), offset: last_offset+10, size: 2});
    header.add_location(Location {name: "sh_num".to_owned(), offset: last_offset+12, size: 2});
    header.add_location(Location {name: "sh_str_index".to_owned(), offset: last_offset+14, size: 2});

    let ph_entry_size = match read_u16(data, last_offset+6) {
        Some(v) => v as usize,
        None => return Err("ELF header is truncated!".to_owned()),
    };

    let ph_num = match read_u16(data, last_offset+8) {
        Some(v) => v as usize,
        None => return Err("ELF header is truncated!".to_owned()),
    };

    let sh_entry_size = match read_u16(data, last_offset+10) {
        Some(v) => v as usize,
        None => return Err("ELF header is truncated!".to_owned()),
    };

    let sh_num= match read_u16(data, last_offset+12) {
        Some(v) => v as usize,
        None => return Err("ELF header is truncated!".to_owned()),
    };

    let sh_str_index = match read_u16(data, last_offset+14) {
        Some(v) => v as usize,
        None => return Err("ELF header is truncated!".to_owned()),
    };

    // program header
    last_offset = ph_offset;
    let mut prog_headers = Vec::<ProgramHeader>::with_capacity(ph_num);

    for segment_index in 0..ph_num {
        header.add_location(Location {name: format!("segment_{segment_index}"), offset: last_offset, size: 0});
        header.add_location(Location {name: ".p_type".to_owned(), offset: last_offset, size: 4});

        let (p_offset, v_offset, v_size) = if elf32 {
            header.add_location(Location {name: ".offset".to_owned(), offset: last_offset+4, size: 4});
            header.add_location(Location {name: ".v_address".to_owned(), offset: last_offset+8, size: 4});
            header.add_location(Location {name: ".p_address".to_owned(), offset: last_offset+12, size: 4});
            header.add_location(Location {name: ".file_size".to_owned(), offset: last_offset+16, size: 4});
            header.add_location(Location {name: ".mem_size".to_owned(), offset: last_offset+20, size: 4});
            header.add_location(Location {name: ".flags".to_owned(), offset: last_offset+24, size: 4});
            header.add_location(Location {name: ".align".to_owned(), offset: last_offset+28, size: 4});

            let offset = match read_u32(data, last_offset+4) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };
            // let size = u32::from_le_bytes(data[last_offset+16..last_offset+20].try_into().unwrap()) as usize;
            header.add_location(Location {name: "data".to_owned(), offset, size: 0});

            let v_offset = match read_u32(data, last_offset+8) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };

            let p_offset = match read_u32(data, last_offset+12) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };

            let v_size = match read_u32(data, last_offset+20) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };

            (p_offset, v_offset, v_size)

        } else {
            header.add_location(Location {name: ".flags".to_owned(), offset: last_offset+4, size: 4});
            header.add_location(Location {name: ".offset".to_owned(), offset: last_offset+8, size: 8});
            header.add_location(Location {name: ".v_address".to_owned(), offset: last_offset+16, size: 8});
            header.add_location(Location {name: ".p_address".to_owned(), offset: last_offset+24, size: 8});
            header.add_location(Location {name: ".file_size".to_owned(), offset: last_offset+32, size: 8});
            header.add_location(Location {name: ".mem_size".to_owned(), offset: last_offset+40, size: 8});
            header.add_location(Location {name: ".align".to_owned(), offset: last_offset+48, size: 8});

            let offset = match read_u64(data, last_offset+8) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };
            // let size = u64::from_le_bytes(data[last_offset+32..last_offset+40].try_into().unwrap()) as usize;
            header.add_location(Location {name: ".data".to_owned(), offset, size: 0});

            let v_offset = match read_u64(data, last_offset+16) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };

            let p_offset = match read_u64(data, last_offset+24) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };

            let v_size = match read_u64(data, last_offset+40) {
                Some(v) => v as usize,
                None => return Err("ELF program header is truncated!".to_owned()),
            };

            (p_offset, v_offset, v_size)
        };

        last_offset += ph_entry_size;
        prog_headers.push(ProgramHeader {p_offset, v_offset, v_size});
    }

    //helper closure to translate virtual address to file offset
    let va_to_file_offset = |va: usize| {
        for segment in &prog_headers {
            if segment.v_offset <= va && (segment.v_offset + segment.v_size) >= va {
                return va - segment.v_offset + segment.p_offset;
            }
        }
        0
    };

    //now we can find valid entry point in the file.
    header[12].offset = va_to_file_offset(entry_point);

    //find section with name table
    let str_section = sh_str_index * sh_entry_size + sh_offset;
    let section_names_table = if elf32 {
        match read_u32(data, str_section+16) {
            Some(v) => v as usize,
            None => return Err("ELF section header is truncated!".to_owned()),
        }
    } else {
        match read_u64(data, str_section+24) {
            Some(v) => v as usize,
            None => return Err("ELF section header is truncated!".to_owned()),
        }
    };

    // section header
    last_offset = sh_offset;

    for section_index in 0..sh_num {
        header.add_location(Location {name: format!("section_{section_index}"), offset: last_offset, size: 0});

        let name_offset = match read_u32(data, last_offset) {
            Some(v) => v as usize,
            None => return Err("ELF section header is truncated!".to_owned()),
        };

        let sec_name = match string_from_u8(data, section_names_table+name_offset) {
            Some(s) => s,
            _ => "name".to_owned(),
        };

        header.add_location(Location {name: sec_name, offset: last_offset, size: 4});
        header.add_location(Location {name: ".type".to_owned(), offset: last_offset+4, size: 4});

        if elf32 {
            header.add_location(Location {name: ".flags".to_owned(), offset: last_offset+8, size: 4});
            header.add_location(Location {name: ".address".to_owned(), offset: last_offset+12, size: 4});
            header.add_location(Location {name: ".offset".to_owned(), offset: last_offset+16, size: 4});
            header.add_location(Location {name: ".size".to_owned(), offset: last_offset+20, size: 4});
            header.add_location(Location {name: ".link".to_owned(), offset: last_offset+24, size: 4});
            header.add_location(Location {name: ".info".to_owned(), offset: last_offset+28, size: 4});
            header.add_location(Location {name: ".align".to_owned(), offset: last_offset+32, size: 4});
            header.add_location(Location {name: ".ent_size".to_owned(), offset: last_offset+36, size: 4});

            let offset = match read_u32(data, last_offset+16) {
                Some(v) => v as usize,
                None => return Err("ELF section header is truncated!".to_owned()),
            };
            header.add_location(Location {name: ".data".to_owned(), offset, size: 0});

        } else {
            header.add_location(Location {name: ".flags".to_owned(), offset: last_offset+8, size: 8});
            header.add_location(Location {name: ".address".to_owned(), offset: last_offset+16, size: 8});
            header.add_location(Location {name: ".offset".to_owned(), offset: last_offset+24, size: 8});
            header.add_location(Location {name: ".size".to_owned(), offset: last_offset+32, size: 8});
            header.add_location(Location {name: ".link".to_owned(), offset: last_offset+36, size: 4});
            header.add_location(Location {name: ".info".to_owned(), offset: last_offset+40, size: 4});
            header.add_location(Location {name: ".align".to_owned(), offset: last_offset+48, size: 8});
            header.add_location(Location {name: ".ent_size".to_owned(), offset: last_offset+56, size: 8});

            let offset = match read_u64(data, last_offset+24) {
                Some(v) => v as usize,
                None => return Err("ELF section header is truncated!".to_owned()),
            };
            header.add_location(Location {name: ".data".to_owned(), offset, size: 0});
        };
        last_offset += sh_entry_size;
    }
    Ok(header)
}
