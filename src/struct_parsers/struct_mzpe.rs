use crate::signatures::is_signature;
use crate::struct_parsers::FieldDescription;

pub fn parse_mzpe_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    let mut mz_header = match parse_mz_struct(data) {
        Err(s) => return Err(s),
        Ok(mzh) => mzh,
    };

    let mut pe_header = match parse_pe_struct(data) {
        Err(s) => return Err(s),
        Ok(peh) => peh,
    };

    mz_header.append(&mut pe_header);
    Ok(mz_header)
}

pub fn parse_mz_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if data.len() < 64 {
        return Err("Too small for 'MZ' header!".to_owned());
    }

    if data[0] != 0x4D || data[1] != 0x5A {
        return Err("Invalid 'MZ' signature!".to_owned());
    }

    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "-- MZ --".to_owned(), offset: 0, size: 0});
    header.push(FieldDescription {name: "magic".to_owned(), offset: 0, size: 2});
    header.push(FieldDescription {name: "bytes_lpage".to_owned(), offset: 2, size: 2});
    header.push(FieldDescription {name: "pages".to_owned(), offset: 4, size: 2});
    header.push(FieldDescription {name: "relocations".to_owned(), offset: 6, size: 2});
    header.push(FieldDescription {name: "header_size".to_owned(), offset: 8, size: 2});
    header.push(FieldDescription {name: "min_alloc".to_owned(), offset: 10, size: 2});
    header.push(FieldDescription {name: "max_alloc".to_owned(), offset: 12, size: 2});
    header.push(FieldDescription {name: "init_ss".to_owned(), offset: 14, size: 2});
    header.push(FieldDescription {name: "init_sp".to_owned(), offset: 16, size: 2});
    header.push(FieldDescription {name: "checksum".to_owned(), offset: 18, size: 2});
    header.push(FieldDescription {name: "init_ip".to_owned(), offset: 20, size: 2});
    header.push(FieldDescription {name: "init_cs".to_owned(), offset: 22, size: 2});
    header.push(FieldDescription {name: "reloc_table".to_owned(), offset: 24, size: 2});
    header.push(FieldDescription {name: "overlay_num".to_owned(), offset: 26, size: 2});
    header.push(FieldDescription {name: "reserved".to_owned(), offset: 28, size: 8});
    header.push(FieldDescription {name: "oem_id".to_owned(), offset: 36, size: 2});
    header.push(FieldDescription {name: "oem_info".to_owned(), offset: 38, size: 2});
    header.push(FieldDescription {name: "reserved".to_owned(), offset: 40, size: 20});
    header.push(FieldDescription {name: "PE_offset".to_owned(), offset: 60, size: 4});
    //TODO: find and parse rich header

    let pe_offset = u32::from_le_bytes(data[60..64].try_into().unwrap()) as usize;
    let header_size = u16::from_le_bytes(data[8..10].try_into().unwrap()) as usize * 16;
    header.push(FieldDescription {name: "dos_stub".to_owned(), offset: header_size, size: pe_offset.saturating_sub(header_size)});

    Ok(header)
}

//https://learn.microsoft.com/en-us/windows/win32/debug/pe-format
pub fn parse_pe_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "mzpe") {
        return Err("Invalid 'MZ_PE' signature!".to_owned());
    }

    //this size is checked in "mzpe" signature
    let pe_offset = u32::from_le_bytes(data[60..64].try_into().unwrap()) as usize;

    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "-- PE --".to_owned(), offset: pe_offset , size: 0});
    header.push(FieldDescription {name: "magic".to_owned(), offset: pe_offset , size: 4});
    header.push(FieldDescription {name: "machine".to_owned(), offset: pe_offset+4, size: 2});
    header.push(FieldDescription {name: "sections".to_owned(), offset: pe_offset+6, size: 2});
    header.push(FieldDescription {name: "time_stamp".to_owned(), offset: pe_offset+8, size: 4});
    header.push(FieldDescription {name: "symbol_table_offset".to_owned(), offset: pe_offset+12, size: 4});
    header.push(FieldDescription {name: "symbols".to_owned(), offset: pe_offset+16, size: 4});
    header.push(FieldDescription {name: "opt_header_size".to_owned(), offset: pe_offset+20, size: 2});
    header.push(FieldDescription {name: "attributes".to_owned(), offset: pe_offset+22, size: 2});

    if data.len() < pe_offset + 136 {
        return Err("PE header seems to be truncated".to_owned());
    }

    //read IMAGE_OPTIONAL_HEADER
    let pe32 = u16::from_le_bytes(data[(pe_offset+24)..(pe_offset+26)].try_into().unwrap()) == 0x010B;
    header.push(FieldDescription {name: (if pe32 {"-- OPT_32 --"} else { "-- OPT_32+ --" }).to_owned(), offset: pe_offset+24, size: 0});
    header.push(FieldDescription {name: "magic".to_owned(), offset: pe_offset+24, size: 2});
    header.push(FieldDescription {name: "linker_major".to_owned(), offset: pe_offset+26, size: 1});
    header.push(FieldDescription {name: "linker_minor".to_owned(), offset: pe_offset+27, size: 1});
    header.push(FieldDescription {name: "code_size".to_owned(), offset: pe_offset+28, size: 4});
    header.push(FieldDescription {name: "init_data_size".to_owned(), offset: pe_offset+32, size: 4});
    header.push(FieldDescription {name: "uninit_data_size".to_owned(), offset: pe_offset+36, size: 4});
    header.push(FieldDescription {name: "entry_point_rva".to_owned(), offset: pe_offset+40, size: 4});
    let entry_point_idx = header.len();
    header.push(FieldDescription {name: "entry_point".to_owned(), offset: 0, size: 0});
    header.push(FieldDescription {name: "code_base".to_owned(), offset: pe_offset+44, size: 4});

    let mut last_offset: usize = if pe32 {
        header.push(FieldDescription {name: "data_base".to_owned(), offset: pe_offset+48, size: 4});
        header.push(FieldDescription {name: "image_base".to_owned(), offset: pe_offset+52, size: 4});
        header.push(FieldDescription {name: "section_align".to_owned(), offset: pe_offset+56, size: 4});
        header.push(FieldDescription {name: "file_align".to_owned(), offset: pe_offset+60, size: 4});
        header.push(FieldDescription {name: "os_major".to_owned(), offset: pe_offset+64, size: 2});
        header.push(FieldDescription {name: "os_minor".to_owned(), offset: pe_offset+66, size: 2});
        header.push(FieldDescription {name: "image_major".to_owned(), offset: pe_offset+68, size: 2});
        header.push(FieldDescription {name: "image_minor".to_owned(), offset: pe_offset+70, size: 2});
        header.push(FieldDescription {name: "subsystem_major".to_owned(), offset: pe_offset+72, size: 2});
        header.push(FieldDescription {name: "subsystem_minor".to_owned(), offset: pe_offset+74, size: 2});
        header.push(FieldDescription {name: "win32_ver".to_owned(), offset: pe_offset+76, size: 4});
        header.push(FieldDescription {name: "image_size".to_owned(), offset: pe_offset+80, size: 4});
        header.push(FieldDescription {name: "headers_size".to_owned(), offset: pe_offset+84, size: 4});
        header.push(FieldDescription {name: "checksum".to_owned(), offset: pe_offset+88, size: 4});
        header.push(FieldDescription {name: "subsystem".to_owned(), offset: pe_offset+92, size: 2});
        header.push(FieldDescription {name: "dll_characteristics".to_owned(), offset: pe_offset+94, size: 2});
        header.push(FieldDescription {name: "stack_reserve_size".to_owned(), offset: pe_offset+96, size: 4});
        header.push(FieldDescription {name: "stack_commit_size".to_owned(), offset: pe_offset+100, size: 4});
        header.push(FieldDescription {name: "heap_reserve_size".to_owned(), offset: pe_offset+104, size: 4});
        header.push(FieldDescription {name: "heap_commit_size".to_owned(), offset: pe_offset+108, size: 4});
        header.push(FieldDescription {name: "loader_flags".to_owned(), offset: pe_offset+112, size: 4});
        header.push(FieldDescription {name: "data_dir_size".to_owned(), offset: pe_offset+116, size: 4});
        pe_offset + 120
    } else {
        header.push(FieldDescription {name: "image_base".to_owned(), offset: pe_offset+48, size: 8});
        header.push(FieldDescription {name: "section_align".to_owned(), offset: pe_offset+56, size: 4});
        header.push(FieldDescription {name: "file_align".to_owned(), offset: pe_offset+60, size: 4});
        header.push(FieldDescription {name: "os_major".to_owned(), offset: pe_offset+64, size: 2});
        header.push(FieldDescription {name: "os_minor".to_owned(), offset: pe_offset+66, size: 2});
        header.push(FieldDescription {name: "image_major".to_owned(), offset: pe_offset+68, size: 2});
        header.push(FieldDescription {name: "image_minor".to_owned(), offset: pe_offset+70, size: 2});
        header.push(FieldDescription {name: "subsystem_major".to_owned(), offset: pe_offset+72, size: 2});
        header.push(FieldDescription {name: "subsystem_minor".to_owned(), offset: pe_offset+74, size: 2});
        header.push(FieldDescription {name: "win32_ver".to_owned(), offset: pe_offset+76, size: 4});
        header.push(FieldDescription {name: "image_size".to_owned(), offset: pe_offset+80, size: 4});
        header.push(FieldDescription {name: "headers_size".to_owned(), offset: pe_offset+84, size: 4});
        header.push(FieldDescription {name: "checksum".to_owned(), offset: pe_offset+88, size: 4});
        header.push(FieldDescription {name: "subsystem".to_owned(), offset: pe_offset+92, size: 2});
        header.push(FieldDescription {name: "dll_characteristics".to_owned(), offset: pe_offset+94, size: 2});
        header.push(FieldDescription {name: "stack_reserve_size".to_owned(), offset: pe_offset+96, size: 8});
        header.push(FieldDescription {name: "stack_commit_size".to_owned(), offset: pe_offset+104, size: 8});
        header.push(FieldDescription {name: "heap_reserve_size".to_owned(), offset: pe_offset+112, size: 8});
        header.push(FieldDescription {name: "heap_commit_size".to_owned(), offset: pe_offset+120, size: 8});
        header.push(FieldDescription {name: "loader_flags".to_owned(), offset: pe_offset+128, size: 4});
        header.push(FieldDescription {name: "data_dir_size".to_owned(), offset: pe_offset+132, size: 4});
        pe_offset + 136
    };

    //TODO: check for data size!
    let data_dir_size = u32::from_le_bytes(data[(last_offset-4)..last_offset].try_into().unwrap()) as usize;
    if data_dir_size > 16 {
        return Err("Too large value in IMAGE_DATA_DIRECTORY.size!".to_owned());
    }

    let data_dir_names = ["export_table", "import_table", "resource_table", "exception_table", "certificate_table",
                        "base_reloc_table", "debug", "architecture", "global_ptr", "tls_table", "load_config",
                        "bount_import", "import_adr_table", "delay_import", "clr_rutine", "reserved"];

    //IMAGE_DATA_DIRECTORY
    header.push(FieldDescription {name: "-- DATA_DIR --".to_owned(), offset: last_offset, size: 0});
    for i in 0..data_dir_size {
        header.push(FieldDescription {name: data_dir_names[i].to_owned(), offset: last_offset, size: 4});
        header.push(FieldDescription {name: "size".to_owned(), offset: last_offset+4, size: 4});
        last_offset += 8;
    }

    //section table
    let entry_point_rva = u32::from_le_bytes(data[(pe_offset+40)..(pe_offset+44)].try_into().unwrap()) as usize;
    let opt_header_size = u16::from_le_bytes(data[(pe_offset+20)..(pe_offset+22)].try_into().unwrap()) as usize;
    let section_count = u16::from_le_bytes(data[(pe_offset+6)..(pe_offset+8)].try_into().unwrap()) as usize;
    last_offset = pe_offset + opt_header_size + 24;

    header.push(FieldDescription {name: "-- SECTIONS --".to_owned(), offset: last_offset, size: 0});
    for _ in 0..section_count {
        //TODO: read and use section name if is good enough
        header.push(FieldDescription {name: "name".to_owned(), offset: last_offset, size: 8});
        header.push(FieldDescription {name: "virtual_size".to_owned(), offset: last_offset+8, size: 4});
        header.push(FieldDescription {name: "virtual_address".to_owned(), offset: last_offset+12, size: 4});
        header.push(FieldDescription {name: "raw_data_size".to_owned(), offset: last_offset+16, size: 4});
        header.push(FieldDescription {name: "raw_data_ptr".to_owned(), offset: last_offset+20, size: 4});
        header.push(FieldDescription {name: "relocation_ptr".to_owned(), offset: last_offset+24, size: 4});
        header.push(FieldDescription {name: "line_num_ptr".to_owned(), offset: last_offset+28, size: 4});
        header.push(FieldDescription {name: "relocation_num".to_owned(), offset: last_offset+32, size: 2});
        header.push(FieldDescription {name: "line_num_num".to_owned(), offset: last_offset+34, size: 2});
        header.push(FieldDescription {name: "characteristics".to_owned(), offset: last_offset+36, size: 4});

        if data.len() < last_offset + 24 {
            return Err("Section Table seems to be truncated!".to_owned());
        }

        let data_size = u32::from_le_bytes(data[(last_offset+16)..(last_offset+20)].try_into().unwrap()) as usize;
        if data_size > 0 {
            //TODO: use section name if is OK
            let data_offset = u32::from_le_bytes(data[(last_offset+20)..(last_offset+24)].try_into().unwrap()) as usize;
            header.push(FieldDescription {name: "section_data".to_owned(), offset: data_offset, size: data_size});
        }

        let section_vs = u32::from_le_bytes(data[(last_offset+8)..(last_offset+12)].try_into().unwrap()) as usize;
        let section_rva = u32::from_le_bytes(data[(last_offset+12)..(last_offset+16)].try_into().unwrap()) as usize;

        if section_rva <= entry_point_rva && (section_rva + section_vs) > entry_point_rva {
            header[entry_point_idx].offset = entry_point_rva - section_rva + u32::from_le_bytes(data[(last_offset+20)..(last_offset+24)].try_into().unwrap()) as usize;
        }
        last_offset += 40;
    }

    //TODO: read and parse data dir
    //TODO: import / export table
    //TODO: resources and other fields from data dir
    Ok(header)
}
