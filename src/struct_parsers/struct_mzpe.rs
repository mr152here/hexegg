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

    let mut header = vec![
        FieldDescription {name: "-- MZ --".to_owned(), offset: 0, size: 0},
        FieldDescription {name: "magic".to_owned(), offset: 0, size: 2},
        FieldDescription {name: "bytes_lpage".to_owned(), offset: 2, size: 2},
        FieldDescription {name: "pages".to_owned(), offset: 4, size: 2},
        FieldDescription {name: "relocations".to_owned(), offset: 6, size: 2},
        FieldDescription {name: "header_size".to_owned(), offset: 8, size: 2},
        FieldDescription {name: "min_alloc".to_owned(), offset: 10, size: 2},
        FieldDescription {name: "max_alloc".to_owned(), offset: 12, size: 2},
        FieldDescription {name: "init_ss".to_owned(), offset: 14, size: 2},
        FieldDescription {name: "init_sp".to_owned(), offset: 16, size: 2},
        FieldDescription {name: "checksum".to_owned(), offset: 18, size: 2},
        FieldDescription {name: "init_ip".to_owned(), offset: 20, size: 2},
        FieldDescription {name: "init_cs".to_owned(), offset: 22, size: 2},
        FieldDescription {name: "reloc_table".to_owned(), offset: 24, size: 2},
        FieldDescription {name: "overlay_num".to_owned(), offset: 26, size: 2},
        FieldDescription {name: "reserved".to_owned(), offset: 28, size: 8},
        FieldDescription {name: "oem_id".to_owned(), offset: 36, size: 2},
        FieldDescription {name: "oem_info".to_owned(), offset: 38, size: 2},
        FieldDescription {name: "reserved".to_owned(), offset: 40, size: 20},
        FieldDescription {name: "PE_offset".to_owned(), offset: 60, size: 4}
    ];

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

    //this data access is checked in "mzpe" signature
    let pe_offset = u32::from_le_bytes(data[60..64].try_into().unwrap()) as usize;

    let mut header = vec![
        FieldDescription {name: "-- PE --".to_owned(), offset: pe_offset , size: 0},
        FieldDescription {name: "magic".to_owned(), offset: pe_offset , size: 4},
        FieldDescription {name: "machine".to_owned(), offset: pe_offset+4, size: 2},
        FieldDescription {name: "sections".to_owned(), offset: pe_offset+6, size: 2},
        FieldDescription {name: "time_stamp".to_owned(), offset: pe_offset+8, size: 4},
        FieldDescription {name: "symbol_table_offset".to_owned(), offset: pe_offset+12, size: 4},
        FieldDescription {name: "symbols".to_owned(), offset: pe_offset+16, size: 4},
        FieldDescription {name: "opt_header_size".to_owned(), offset: pe_offset+20, size: 2},
        FieldDescription {name: "attributes".to_owned(), offset: pe_offset+22, size: 2}
    ];

    if data.len() < pe_offset + 136 {
        return Err("PE header seems to be truncated!".to_owned());
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

    let (mut last_offset, image_base) = if pe32 {
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
        (pe_offset + 120, u32::from_le_bytes(data[(pe_offset+52)..(pe_offset+56)].try_into().unwrap()) as usize)
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
        (pe_offset + 136, u64::from_le_bytes(data[(pe_offset+48)..(pe_offset+56)].try_into().unwrap()) as usize)
    };

    let data_dir_size = u32::from_le_bytes(data[(last_offset-4)..last_offset].try_into().unwrap()) as usize;
    if data_dir_size > 16 {
        return Err(format!("IMAGE_DATA_DIRECTORY size is more than 16 at 0x{:08X}!", last_offset-4));

    } else if data.len() < (last_offset + data_dir_size*8) {
        return Err("IMAGE_DATA_DIRECTORY table is truncated!".to_owned());
    }

    //IMAGE_DATA_DIRECTORY
    header.push(FieldDescription {name: "-- DATA_DIR --".to_owned(), offset: last_offset, size: 0});

    let mut data_dir = Vec::<(usize, usize)>::with_capacity(data_dir_size);
    let data_dir_names = ["export_table", "import_table", "resource_table", "exception_table", "certificate_table",
                        "base_reloc_table", "debug", "architecture", "global_ptr", "tls_table", "load_config",
                        "bount_import", "import_adr_table", "delay_import", "clr_rutine", "reserved"];

    for dir_name in data_dir_names.iter().take(data_dir_size) {
        header.push(FieldDescription {name: dir_name.to_string(), offset: last_offset, size: 4});
        header.push(FieldDescription {name: "size".to_owned(), offset: last_offset+4, size: 4});

        let rva = u32::from_le_bytes(data[(last_offset)..(last_offset+4)].try_into().unwrap()) as usize;
        let size = u32::from_le_bytes(data[(last_offset+4)..(last_offset+8)].try_into().unwrap()) as usize;
        data_dir.push((rva,size));

        last_offset += 8;
    }

    //section table
    let opt_header_size = u16::from_le_bytes(data[(pe_offset+20)..(pe_offset+22)].try_into().unwrap()) as usize;
    let section_count = u16::from_le_bytes(data[(pe_offset+6)..(pe_offset+8)].try_into().unwrap()) as usize;
    let mut sections = Vec::<SectionInfo>::with_capacity(section_count);

    last_offset = pe_offset + opt_header_size + 24;

    if data.len() < (last_offset + 40 * section_count) {
        return Err("Section table is truncated!".to_owned());
    }

    header.push(FieldDescription {name: "-- SECTIONS --".to_owned(), offset: last_offset, size: 0});
    for _ in 0..section_count {

        //try to get section name
        let name_len = (0..8).find(|i| u8::from_le_bytes(data[(last_offset + i)..(last_offset + i + 1)].try_into().unwrap()).is_ascii_control()).unwrap_or(8);
        let section_name = if name_len > 0 {
            String::from_utf8_lossy(&data[last_offset..(last_offset+name_len)]).to_string()
        } else {
            "<not_ascii_name>".to_owned()
        };
        header.push(FieldDescription {name: section_name, offset: last_offset, size: 8});
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
            header.push(FieldDescription {name: "raw_data".to_owned(), offset: si.raw_offset, size: si.raw_size});
        }

        sections.push(si);
        last_offset += 40;
    }

    //helper closure that find and translate relative virtual address to file offset
    let rva_to_file_offset = |rva: usize| {
        for section in &sections {
            if section.rva <= rva && (section.rva + section.virtual_size) > rva {
                return rva - section.rva + section.raw_offset;
            }
        }
        0
    };

    //helper closure that find and translate virtual address to file offset
    let va_to_file_offset = |va: usize| {
        rva_to_file_offset(va.saturating_sub(image_base))
    };

    //find and fill entry point
    let entry_point_rva = u32::from_le_bytes(data[(pe_offset+40)..(pe_offset+44)].try_into().unwrap()) as usize;
    header[entry_point_idx].offset = rva_to_file_offset(entry_point_rva);

    //export table
    if let Some(&(export_table_rva, export_table_size)) = data_dir.first() {

        if export_table_rva > 0 && export_table_size > 0 {
            last_offset = rva_to_file_offset(export_table_rva);

            if last_offset == 0 {
                return Err("Export table set but not found in file!".to_owned());

            } else if data.len() < (last_offset + 40) {
                return Err("Export table is truncated!".to_owned());
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
            let name_ptr_table_fo = rva_to_file_offset(name_ptr_table_rva);
            header.push(FieldDescription {name: "name_ptr_table".to_owned(), offset: name_ptr_table_fo, size: 0});

            let ordinal_table_entries = u32::from_le_bytes(data[(last_offset+24)..(last_offset+28)].try_into().unwrap()) as usize;
            let ordinal_table_rva = u32::from_le_bytes(data[(last_offset+36)..(last_offset+40)].try_into().unwrap()) as usize;
            let ordinal_table_fo = rva_to_file_offset(ordinal_table_rva);
            header.push(FieldDescription {name: "ordinal_table".to_owned(), offset: ordinal_table_fo, size: ordinal_table_entries*2});

            let ordinal_base = u32::from_le_bytes(data[(last_offset+16)..(last_offset+20)].try_into().unwrap()) as usize;

            if data.len() < (address_table_offset + 4*address_table_entries) {
                return Err("Export address table is truncated!".to_owned());

            } else if data.len() < (ordinal_table_fo + 2*ordinal_table_entries) {
                return Err("Ordinal table is truncated!".to_owned());
            }

            //iterate over address_table and get every export file offset + its name
            for ordinal in 0..address_table_entries {
                last_offset = address_table_offset + 4*ordinal;
                let export_rva = u32::from_le_bytes(data[last_offset..(last_offset+4)].try_into().unwrap()) as usize;

                if export_rva != 0 {
                    let export_offset = rva_to_file_offset(export_rva);

                    //find it in ordinal table
                    let mut name_idx = None;
                    for idx in 0..ordinal_table_entries {
                        let ordinal_num = u16::from_le_bytes(data[(ordinal_table_fo + 2*idx)..(ordinal_table_fo + 2*(idx+1))].try_into().unwrap()) as usize;
                        if ordinal_num == ordinal {
                            name_idx = Some(idx);
                            break;
                        }
                    }

                    //if it is named export
                    if let Some(name_idx) = name_idx {
                        if data.len() < (name_ptr_table_fo + 4*(name_idx+1)) {
                            return Err("Pointer to export name is out of file range!".to_owned());
                        }

                        let export_name_rva = u32::from_le_bytes(data[(name_ptr_table_fo + 4*name_idx)..(name_ptr_table_fo + 4*(name_idx+1))].try_into().unwrap()) as usize;
                        let export_name_fo = rva_to_file_offset(export_name_rva);
                        let mut export_name = String::new();

                        for i in 0..30 {
                            if data.len() < (export_name_fo + i) {
                                return Err("Export name is out of file range!".to_owned());
                            }

                            if data[export_name_fo + i].is_ascii_control() {
                                break;
                            } else {
                                export_name.push(char::try_from(data[export_name_fo+i]).unwrap_or(' '));
                            }
                        }
                        header.push(FieldDescription {name: format!("{}_{}", ordinal_base+ordinal, export_name), offset: export_offset, size: 0});

                    } else {
                        header.push(FieldDescription {name: format!("{}_<no_name>", ordinal_base+ordinal), offset: export_offset, size: 0});
                    }
                }
            }
        }
    }

    //certificates
    if let Some(&(cert_table_fo, cert_table_size)) = data_dir.get(4) {
        if cert_table_fo > 0 && cert_table_size > 0 {
            last_offset = cert_table_fo;
            header.push(FieldDescription {name: "-- CERTIFICATES --".to_owned(), offset: last_offset, size: 0});

            while last_offset < (cert_table_fo + cert_table_size) && data.len() > (last_offset + 4) {
                let cert_len = u32::from_le_bytes(data[(last_offset)..(last_offset+4)].try_into().unwrap()) as usize;
                header.push(FieldDescription {name: "length".to_owned(), offset: last_offset, size: 4});
                header.push(FieldDescription {name: "revision".to_owned(), offset: last_offset+4, size: 2});
                header.push(FieldDescription {name: "type".to_owned(), offset: last_offset+6, size: 2});
                header.push(FieldDescription {name: "certificate".to_owned(), offset: last_offset+8, size: cert_len - 8});

                //next cert table is at rounded up to 8 offset
                last_offset += (cert_len + 7) & !7;
            }
        }
    }

    //tls table
    if let Some(&(tls_table_rva, tls_table_size)) = data_dir.get(9) {
        if tls_table_rva > 0 && tls_table_size > 0 {
            let tls_fo = rva_to_file_offset(tls_table_rva);
            header.push(FieldDescription {name: "-- TLS --".to_owned(), offset: tls_fo, size: 0});

            let field_size = if pe32 { 4 } else { 8 };
            header.push(FieldDescription {name: "data_start".to_owned(), offset: tls_fo, size: field_size});
            header.push(FieldDescription {name: "data_end".to_owned(), offset: tls_fo + field_size, size: field_size});
            header.push(FieldDescription {name: "index_table".to_owned(), offset: tls_fo + 2*field_size, size: field_size});
            header.push(FieldDescription {name: "callbacks_table".to_owned(), offset: tls_fo + 3*field_size, size: field_size});
            header.push(FieldDescription {name: "zero_fill_size".to_owned(), offset: tls_fo + 4*field_size, size: 4});
            header.push(FieldDescription {name: "characteristics".to_owned(), offset: tls_fo + 4*field_size + 4, size: 4});

            if data.len() < (tls_fo + 4*field_size) {
                return Err("TLS callback table is out of file range!".to_owned());
            }

            let callbacks_va = if pe32 {
                u32::from_le_bytes(data[(tls_fo + 3*field_size)..(tls_fo + 4*field_size)].try_into().unwrap()) as usize
            } else {
                u64::from_le_bytes(data[(tls_fo + 3*field_size)..(tls_fo + 4*field_size)].try_into().unwrap()) as usize
            };

            let callbacks_table_fo = va_to_file_offset(callbacks_va);
            let mut i = 0;
            loop {

                if data.len() < (callbacks_table_fo + (i+1)*field_size) {
                    return Err("TLS callback function is out of file range!".to_owned());
                }

                let callbacks_fn_va = if pe32 {
                    u32::from_le_bytes(data[(callbacks_table_fo + i*field_size)..(callbacks_table_fo + (i+1)*field_size)].try_into().unwrap()) as usize
                } else {
                    u64::from_le_bytes(data[(callbacks_table_fo + i*field_size)..(callbacks_table_fo + (i+1)*field_size)].try_into().unwrap()) as usize
                };

                if callbacks_fn_va == 0 {
                    break;
                }

                let callbacks_fn_fo = va_to_file_offset(callbacks_fn_va);
                header.push(FieldDescription {name: format!("tls_{:08X}", callbacks_fn_va), offset: callbacks_fn_fo, size: 0});
                i += 1;
            }
        }
    }

    Ok(header)
}
