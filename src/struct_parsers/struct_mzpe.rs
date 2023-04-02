use crate::signatures::is_signature;
use crate::struct_parsers::FieldDescription;


struct SectionInfo {
    rva: usize,
    virtual_size: usize,
    raw_offset: usize,
    raw_size: usize
}

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

    let mut cert_table = 0;
    let mut cert_table_size = 0;
    let mut export_table = 0;
    let mut export_table_size = 0;
    let data_dir_names = ["export_table", "import_table", "resource_table", "exception_table", "certificate_table",
                        "base_reloc_table", "debug", "architecture", "global_ptr", "tls_table", "load_config",
                        "bount_import", "import_adr_table", "delay_import", "clr_rutine", "reserved"];

    //IMAGE_DATA_DIRECTORY
    header.push(FieldDescription {name: "-- DATA_DIR --".to_owned(), offset: last_offset, size: 0});
    for i in 0..data_dir_size {
        header.push(FieldDescription {name: data_dir_names[i].to_owned(), offset: last_offset, size: 4});
        header.push(FieldDescription {name: "size".to_owned(), offset: last_offset+4, size: 4});

        if i == 0 {
            export_table = u32::from_le_bytes(data[(last_offset)..(last_offset+4)].try_into().unwrap()) as usize;
            export_table_size = u32::from_le_bytes(data[(last_offset+4)..(last_offset+8)].try_into().unwrap()) as usize;

        } else if i == 4 {
            cert_table = u32::from_le_bytes(data[(last_offset)..(last_offset+4)].try_into().unwrap()) as usize;
            cert_table_size = u32::from_le_bytes(data[(last_offset+4)..(last_offset+8)].try_into().unwrap()) as usize;
        }

        last_offset += 8;
    }

    //section table
    let opt_header_size = u16::from_le_bytes(data[(pe_offset+20)..(pe_offset+22)].try_into().unwrap()) as usize;
    let section_count = u16::from_le_bytes(data[(pe_offset+6)..(pe_offset+8)].try_into().unwrap()) as usize;
    let mut sections = Vec::<SectionInfo>::with_capacity(section_count);

    last_offset = pe_offset + opt_header_size + 24;

    header.push(FieldDescription {name: "-- SECTIONS --".to_owned(), offset: last_offset, size: 0});
    for _ in 0..section_count {

        if data.len() < last_offset + 40 {
            return Err("Section Table seems to be truncated!".to_owned());
        }
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

        //fill section table
        let si = SectionInfo {
            rva: u32::from_le_bytes(data[(last_offset+12)..(last_offset+16)].try_into().unwrap()) as usize,
            virtual_size: u32::from_le_bytes(data[(last_offset+8)..(last_offset+12)].try_into().unwrap()) as usize,
            raw_offset: u32::from_le_bytes(data[(last_offset+20)..(last_offset+24)].try_into().unwrap()) as usize,
            raw_size: u32::from_le_bytes(data[(last_offset+16)..(last_offset+20)].try_into().unwrap()) as usize,
        };

        //add section data (if any)
        if si.raw_size > 0 {
            //TODO: use section name if is OK
            header.push(FieldDescription {name: "section_data".to_owned(), offset: si.raw_offset, size: si.raw_size});
        }

        sections.push(si);
        last_offset += 40;
    }

    //closure that find and translate relative virtual address to file offset
    let rva_to_file_offset = |rva| {
        for section in &sections {
            if section.rva <= rva && (section.rva + section.virtual_size) > rva {
                return rva - section.rva + section.raw_offset;
            }
        }
        0
    };

    //find and fill entry point
    let entry_point_rva = u32::from_le_bytes(data[(pe_offset+40)..(pe_offset+44)].try_into().unwrap()) as usize;
    header[entry_point_idx].offset = rva_to_file_offset(entry_point_rva);

    //export table
    if export_table > 0 && export_table_size > 0 {
        last_offset = rva_to_file_offset(export_table);
        if last_offset == 0 {
            return Err("Export table not found in any section!".to_owned());
        }

        header.push(FieldDescription {name: "-- EXPORTS --".to_owned(), offset: last_offset, size: 0});
        header.push(FieldDescription {name: "flags".to_owned(), offset: last_offset, size: 4});
        header.push(FieldDescription {name: "time_stamp".to_owned(), offset: last_offset+4, size: 4});
        header.push(FieldDescription {name: "major".to_owned(), offset: last_offset+8, size: 2});
        header.push(FieldDescription {name: "minor".to_owned(), offset: last_offset+10, size: 2});
        header.push(FieldDescription {name: "name_rva".to_owned(), offset: last_offset+12, size: 4});
        header.push(FieldDescription {name: "ordinal_base".to_owned(), offset: last_offset+16, size: 4});
        header.push(FieldDescription {name: "address_table_entries".to_owned(), offset: last_offset+20, size: 4});
        header.push(FieldDescription {name: "name_pointer_count".to_owned(), offset: last_offset+24, size: 4});
        header.push(FieldDescription {name: "export_table_rva".to_owned(), offset: last_offset+28, size: 4});
        header.push(FieldDescription {name: "name_pointer_rva".to_owned(), offset: last_offset+32, size: 4});
        header.push(FieldDescription {name: "ordinal_table_rva".to_owned(), offset: last_offset+36, size: 4});


        let address_table_entries = u32::from_le_bytes(data[(last_offset+20)..(last_offset+24)].try_into().unwrap()) as usize;
        let address_table_rva = u32::from_le_bytes(data[(last_offset+28)..(last_offset+32)].try_into().unwrap()) as usize;
        let address_table_offset = rva_to_file_offset(address_table_rva);
        header.push(FieldDescription {name: "address_table".to_owned(), offset: address_table_offset, size: address_table_entries*4});

        let name_ptr_table_rva = u32::from_le_bytes(data[(last_offset+32)..(last_offset+36)].try_into().unwrap()) as usize;
        let name_ptr_table_offset = rva_to_file_offset(name_ptr_table_rva);
        header.push(FieldDescription {name: "name_ptr_table".to_owned(), offset: name_ptr_table_offset, size: 0});

        let ordinal_table_entries = u32::from_le_bytes(data[(last_offset+24)..(last_offset+28)].try_into().unwrap()) as usize;
        let ordinal_table_rva = u32::from_le_bytes(data[(last_offset+36)..(last_offset+40)].try_into().unwrap()) as usize;
        let ordinal_table_offset = rva_to_file_offset(ordinal_table_rva);
        header.push(FieldDescription {name: "ordinal_table".to_owned(), offset: ordinal_table_offset, size: ordinal_table_entries*2});

        let ordinal_base = u32::from_le_bytes(data[(last_offset+16)..(last_offset+20)].try_into().unwrap()) as usize;

        //iterate over address_table and get every export file offset + its name
        for ordinal in 0..address_table_entries {
            last_offset = address_table_offset + 4*ordinal;
            let export_rva = u32::from_le_bytes(data[last_offset..(last_offset+4)].try_into().unwrap()) as usize;

            if export_rva != 0 {
                let export_offset = rva_to_file_offset(export_rva);

                //find it in ordinal table
                let mut name_idx = None;
                for idx in 0..ordinal_table_entries {
                    let ordinal_num = u16::from_le_bytes(data[(ordinal_table_offset + 2*idx)..(ordinal_table_offset + 2*(idx+1))].try_into().unwrap()) as usize;
                    if ordinal_num == ordinal {
                        name_idx = Some(idx);
                        break;
                    }
                }

                //if it is named export
                if let Some(nidx) = name_idx {
                    let export_name_rva = u32::from_le_bytes(data[(name_ptr_table_offset + 4*nidx)..(name_ptr_table_offset + 4*(nidx+1))].try_into().unwrap()) as usize;
                    let export_name_offset = rva_to_file_offset(export_name_rva);

                    let mut name_len = 30;
                    for i in 0..30 {
                        if u8::from_le_bytes(data[(export_name_offset + i)..(export_name_offset + i+1)].try_into().unwrap()).is_ascii_control() {
                            name_len = i;
                            break;
                        }
                    }

                    let export_name = String::from_utf8_lossy(&data[export_name_offset..export_name_offset+name_len]).to_string();
                    header.push(FieldDescription {name: format!("{}_{}", ordinal_base+ordinal, export_name), offset: export_offset, size: 0});
                } else {
                    header.push(FieldDescription {name: format!("{}_<no_name>", ordinal_base+ordinal), offset: export_offset, size: 0});
                }
            }
        }
    }

    //certificates
    if cert_table > 0 && cert_table_size > 0 {
        last_offset = cert_table;
        header.push(FieldDescription {name: "-- CERTIFICATES --".to_owned(), offset: last_offset, size: 0});

        while last_offset < (cert_table + cert_table_size) && data.len() > (last_offset + 4) {
            let cert_len = u32::from_le_bytes(data[(last_offset)..(last_offset+4)].try_into().unwrap()) as usize;
            header.push(FieldDescription {name: "length".to_owned(), offset: last_offset, size: 4});
            header.push(FieldDescription {name: "revision".to_owned(), offset: last_offset+4, size: 2});
            header.push(FieldDescription {name: "type".to_owned(), offset: last_offset+6, size: 2});
            header.push(FieldDescription {name: "certificate".to_owned(), offset: last_offset+8, size: cert_len - 8});
            //next cert table is at rounded up to 8 offset
            last_offset += cert_len + 7 & !7;
        }
    }


    //TODO: read and parse data dir
    //TODO: import / export table
    //TODO: resources and other fields from data dir
    Ok(header)
}
