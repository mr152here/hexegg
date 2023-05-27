use crate::signatures::is_signature;
use crate::struct_parsers::*;

type Read16u = dyn Fn(&[u8], usize) -> Option<u16>;
type Read32u = dyn Fn(&[u8], usize) -> Option<u32>;

pub fn parse_pcapng_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "pcapng") {
        return Err("Invalid 'PCAPNG' signature!".to_owned());
    }

    //read section header block
    let mut header = vec![
        FieldDescription {name: "-- PCAPNG --".to_owned(), offset: 0, size: 0},
        FieldDescription {name: "block_type".to_owned(), offset: 0, size: 4},
        FieldDescription {name: "block_length".to_owned(), offset: 4, size: 4},
        FieldDescription {name: "byte_order".to_owned(), offset: 8, size: 4},
        FieldDescription {name: "major".to_owned(), offset: 12, size: 2},
        FieldDescription {name: "minor".to_owned(), offset: 14, size: 2},
        FieldDescription {name: "section_length".to_owned(), offset: 16, size: 8},
    ];

    let little_endian = match read_le_u32(data, 8) {
        Some(v) if v == 0x1a2b3c4d => true,
        Some(v) if v == 0x4d3c2b1a => false,
        _ => return Err("Invalid 'byte_order' value!".to_owned()),
    };

    let read_u16 = if little_endian { read_le_u16 } else { read_be_u16 };
    let read_u32 = if little_endian { read_le_u32 } else { read_be_u32 };
    let read_u64 = if little_endian { read_le_u64 } else { read_be_u64 };

    let block_length = match read_u32(data, 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let section_length = match read_u64(data, 16) {
        Some(v) => v as usize,
        None => return Err("'section_length' is out of data range!".to_owned()),
    };

    //parse options (if any)
    if block_length > 28 {
        match parse_options(data, 24, &read_u16) {
            Ok(options) => header.extend(options),
            Err(s) => return Err(s),
        }
    }

    header.push(FieldDescription {name: "block_length".to_owned(), offset: block_length - 4, size: 4});

    //read in the loop all other blocks for current section until
    //next section header is found or EOF or section_length is reached
    let mut offset = block_length;
    while offset < data.len() && offset < section_length {

        //get block type
        if let Some(block_type) = read_u32(data, offset) {
            let block = match block_type {
                0x1 => interface_description(data, offset, &read_u16, &read_u32),
                0x3 => simple_packet(data, offset, &read_u32),
                0x4 => name_resolution(data, offset, &read_u16, &read_u32),
                0x5 => interface_statistics(data, offset, &read_u16, &read_u32),
                0x6 => enhanced_packet(data, offset, &read_u16, &read_u32),
                0xA => decryption_secrets(data, offset, &read_u16, &read_u32),
                0x0A0D0D0A => break,
                _ => unknown_block(data, offset, &read_u32),
            };

            match block {
                Ok(bl) => header.extend(bl),
                Err(s) => return Err(s),
            }
        } else {
            return Err("'block_type' is out of data range!".to_owned());
        }

        let block_length = match read_u32(data, offset + 4) {
            Some(v) => v as usize,
            None => return Err("'block_length' is out of data range!".to_owned()),
        };
        offset += block_length;
    }

    Ok(header)
}

fn parse_options(data: &[u8], mut offset: usize, read_u16: &Read16u) -> Result<Vec<FieldDescription>, String> {

    let mut options = vec![FieldDescription {name: "options".to_owned(), offset, size: 2}];

    loop {
        let option_code = match read_u16(data, offset) {
            Some(v) => v,
            None => return Err("'option_code' is out of data range!".to_owned()),
        };

        let option_length = match read_u16(data, offset + 2) {
            Some(v) => v,
            None => return Err("'option_size' is out of data range!".to_owned()),
        };

        options.push(FieldDescription {name: ".code".to_owned(), offset, size: 2});
        options.push(FieldDescription {name: ".length".to_owned(), offset: offset + 2, size: 2});

        if option_length > 0 {
            options.push(FieldDescription {name: ".data".to_owned(), offset: offset + 4, size: option_length as usize});
        }

        //align length to 4 bytes
        let option_aligned_length = (option_length + 3) & !3;
        offset += 4 + option_aligned_length as usize;

        //if it was opt_endofopt, break
        if option_code == 0 {
            break;
        }
    }

    Ok(options)
}

fn parse_records(data: &[u8], mut offset: usize, read_u16: &Read16u) -> Result<Vec<FieldDescription>, String> {

    let mut records = vec![FieldDescription {name: "records".to_owned(), offset, size: 2}];

    loop {
        let record_type = match read_u16(data, offset) {
            Some(v) => v,
            None => return Err("'record_type' is out of data range!".to_owned()),
        };

        let record_length = match read_u16(data, offset + 2) {
            Some(v) => v,
            None => return Err("'record_length' is out of data range!".to_owned()),
        };

        records.push(FieldDescription {name: ".type".to_owned(), offset, size: 2});
        records.push(FieldDescription {name: ".length".to_owned(), offset: offset + 2, size: 2});

        if record_length > 0 {
            records.push(FieldDescription {name: ".data".to_owned(), offset: offset + 4, size: record_length as usize});
        }

        //align length to 4 bytes
        let aligned_length = (record_length + 3) & !3;
        offset += 4 + aligned_length as usize;

        //if it was nrb_record_end, break
        if record_type == 0 {
            break;
        }
    }
    Ok(records)
}

//process interface description block
fn interface_description(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let mut block = vec![
        FieldDescription {name: "interface_desc_block".to_owned(), offset, size: 4},
        FieldDescription {name: ".length".to_owned(), offset: offset + 4, size: 4},
        FieldDescription {name: ".link_type".to_owned(), offset: offset + 8, size: 2},
        FieldDescription {name: ".reserved".to_owned(), offset: offset + 10, size: 2},
        FieldDescription {name: ".snap_length".to_owned(), offset: offset + 12, size: 4},
    ];

    if block_length > 20 {
        match parse_options(data, offset + 16, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                options.iter_mut().for_each(|fd| {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(fd.name.as_str());
                    fd.name = new_name;
                });
                block.extend(options)
            },
        }
    }

    block.push(FieldDescription {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

//process simple packet block
fn simple_packet(data: &[u8], offset: usize, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let packet_length = match read_u32(data, offset + 8) {
        Some(v) => v as usize,
        None => return Err("'packet_length' is out of data range!".to_owned()),
    };

    Ok(vec![
        FieldDescription {name: "simple_packet_block".to_owned(), offset, size: 4},
        FieldDescription {name: ".length".to_owned(), offset: offset + 4, size: 4},
        FieldDescription {name: ".original_length".to_owned(), offset: offset + 8, size: 4},
        FieldDescription {name: ".packet_data".to_owned(), offset: offset + 12, size: packet_length},
        FieldDescription {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4},
    ])
}

//process name resolution block
fn name_resolution(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let mut block = vec![
        FieldDescription {name: "name_resolution_block".to_owned(), offset, size: 4},
        FieldDescription {name: ".length".to_owned(), offset: offset + 4, size: 4}
    ];

    //parse records
    match parse_records(data, offset + 8, read_u16) {
        Err(s) => return Err(s),
        Ok(mut records) => {
            records.iter_mut().for_each(|fd| {
                let mut new_name = ".".to_owned();
                new_name.push_str(fd.name.as_str());
                fd.name = new_name;
            });
            block.extend(records)
        },
    }

    //parse options
    let last_offset = block.last().unwrap().offset + 2;

    if block_length > (last_offset + 4) {
        match parse_options(data, last_offset, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                options.iter_mut().for_each(|fd| {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(fd.name.as_str());
                    fd.name = new_name;
                });
                block.extend(options)
            },
        }
    }

    block.push(FieldDescription {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

fn interface_statistics(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let mut block = vec![
        FieldDescription {name: "interface_statistics_block".to_owned(), offset, size: 4},
        FieldDescription {name: ".length".to_owned(), offset: offset + 4, size: 4},
        FieldDescription {name: ".interface_id".to_owned(), offset: offset + 8, size: 4},
        FieldDescription {name: ".time_stamp_h".to_owned(), offset: offset + 12, size: 4},
        FieldDescription {name: ".time_stamp_l".to_owned(), offset: offset + 16, size: 4},
    ];

    //parse options
    if block_length > 6*4 {
        match parse_options(data, offset + 20, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                options.iter_mut().for_each(|fd| {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(fd.name.as_str());
                    fd.name = new_name;
                });
                block.extend(options)
            },
        }
    }

    block.push(FieldDescription {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

//process enhanced packet block
fn enhanced_packet(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let captured_length = match read_u32(data, offset + 20) {
        Some(v) => v as usize,
        None => return Err("'captured_length' is out of data range!".to_owned()),
    };

    let mut block = vec![
        FieldDescription {name: "enhanced_packet_block".to_owned(), offset, size: 4},
        FieldDescription {name: ".length".to_owned(), offset: offset + 4, size: 4},
        FieldDescription {name: ".interface_id".to_owned(), offset: offset + 8, size: 4},
        FieldDescription {name: ".time_stamp_h".to_owned(), offset: offset + 12, size: 4},
        FieldDescription {name: ".time_stamp_l".to_owned(), offset: offset + 16, size: 4},
        FieldDescription {name: ".captured_length".to_owned(), offset: offset + 20, size: 4},
        FieldDescription {name: ".original_length".to_owned(), offset: offset + 24, size: 4},
        FieldDescription {name: ".data".to_owned(), offset: offset + 28, size: captured_length},
    ];

    let captured_aligned_length = (captured_length + 3) & !3;

    if block_length > (8*4 + captured_aligned_length) {
        match parse_options(data, offset + captured_aligned_length, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                options.iter_mut().for_each(|fd| {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(fd.name.as_str());
                    fd.name = new_name;
                });
                block.extend(options)
            },
        }
    }

    block.push(FieldDescription {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

//process decryption secrets block
fn decryption_secrets(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let secrets_length = match read_u32(data, offset + 12) {
        Some(v) => v as usize,
        None => return Err("'captured_length' is out of data range!".to_owned()),
    };

    let mut block = vec![
        FieldDescription {name: "decryption_secrets_block".to_owned(), offset, size: 4},
        FieldDescription {name: ".length".to_owned(), offset: offset + 4, size: 4},
        FieldDescription {name: ".secrets_type".to_owned(), offset: offset + 8, size: 4},
        FieldDescription {name: ".secrets_length".to_owned(), offset: offset + 12, size: 4},
        FieldDescription {name: ".data".to_owned(), offset: offset + 16, size: secrets_length},
    ];

    let aligned_length = (secrets_length + 3) & !3;
    if block_length > (5*4 + aligned_length) {
        match parse_options(data, offset + aligned_length, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                options.iter_mut().for_each(|fd| {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(fd.name.as_str());
                    fd.name = new_name;
                });
                block.extend(options)
            },
        }
    }

    block.push(FieldDescription {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

fn unknown_block(data: &[u8], offset: usize, read_u32: &Read32u) -> Result<Vec<FieldDescription>, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };
    Ok(vec![FieldDescription {name: "unknown_block".to_owned(), offset, size: block_length}])
}
