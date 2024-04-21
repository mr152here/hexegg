use crate::signatures::is_signature;
use crate::location_list::{Location, LocationList};
use crate::struct_parsers::*;

type Read16u = dyn Fn(&[u8], usize) -> Option<u16>;
type Read32u = dyn Fn(&[u8], usize) -> Option<u32>;

pub fn parse_pcapng_struct(data: &[u8]) -> Result<LocationList, String> {

    if !is_signature(data, "pcapng") {
        return Err("Invalid 'PCAPNG' signature!".to_owned());
    }

    //read section header block
    let mut header = LocationList::new();
    header.add_location(Location {name: "-- PCAPNG --".to_owned(), offset: 0, size: 0});
    header.add_location(Location {name: "block_type".to_owned(), offset: 0, size: 4});
    header.add_location(Location {name: "block_length".to_owned(), offset: 4, size: 4});
    header.add_location(Location {name: "byte_order".to_owned(), offset: 8, size: 4});
    header.add_location(Location {name: "major".to_owned(), offset: 12, size: 2});
    header.add_location(Location {name: "minor".to_owned(), offset: 14, size: 2});
    header.add_location(Location {name: "section_length".to_owned(), offset: 16, size: 8});

    let little_endian = match read_le_u32(data, 8) {
        Some(0x1a2b3c4d) => true,
        Some(0x4d3c2b1a) => false,
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

    header.add_location(Location {name: "block_length".to_owned(), offset: block_length - 4, size: 4});

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

fn parse_options(data: &[u8], mut offset: usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let mut options = LocationList::new();
    options.add_location(Location {name: "options".to_owned(), offset, size: 2});

    loop {
        let option_code = match read_u16(data, offset) {
            Some(v) => v,
            None => return Err("'option_code' is out of data range!".to_owned()),
        };

        let option_length = match read_u16(data, offset + 2) {
            Some(v) => v,
            None => return Err("'option_size' is out of data range!".to_owned()),
        };

        options.add_location(Location {name: ".code".to_owned(), offset, size: 2});
        options.add_location(Location {name: ".length".to_owned(), offset: offset + 2, size: 2});

        if option_length > 0 {
            options.add_location(Location {name: ".data".to_owned(), offset: offset + 4, size: option_length as usize});
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

fn parse_records(data: &[u8], mut offset: usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let mut records = LocationList::new();
    records.add_location(Location {name: "records".to_owned(), offset, size: 2});

    loop {
        let record_type = match read_u16(data, offset) {
            Some(v) => v,
            None => return Err("'record_type' is out of data range!".to_owned()),
        };

        let record_length = match read_u16(data, offset + 2) {
            Some(v) => v,
            None => return Err("'record_length' is out of data range!".to_owned()),
        };

        records.add_location(Location {name: ".type".to_owned(), offset, size: 2});
        records.add_location(Location {name: ".length".to_owned(), offset: offset + 2, size: 2});

        if record_length > 0 {
            records.add_location(Location {name: ".data".to_owned(), offset: offset + 4, size: record_length as usize});
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
fn interface_description(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let mut block = LocationList::new();
    block.add_location(Location {name: "interface_desc_block".to_owned(), offset, size: 4});
    block.add_location(Location {name: ".length".to_owned(), offset: offset + 4, size: 4});
    block.add_location(Location {name: ".link_type".to_owned(), offset: offset + 8, size: 2});
    block.add_location(Location {name: ".reserved".to_owned(), offset: offset + 10, size: 2});
    block.add_location(Location {name: ".snap_length".to_owned(), offset: offset + 12, size: 4});

    if block_length > 20 {
        match parse_options(data, offset + 16, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                for loc in &mut options {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(loc.name.as_str());
                    loc.name = new_name;
                }
                block.extend(options)
            },
        }
    }

    block.add_location(Location {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

//process simple packet block
fn simple_packet(data: &[u8], offset: usize, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let packet_length = match read_u32(data, offset + 8) {
        Some(v) => v as usize,
        None => return Err("'packet_length' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "simple_packet_block".to_owned(), offset, size: 4});
    ll.add_location(Location {name: ".length".to_owned(), offset: offset + 4, size: 4});
    ll.add_location(Location {name: ".original_length".to_owned(), offset: offset + 8, size: 4});
    ll.add_location(Location {name: ".packet_data".to_owned(), offset: offset + 12, size: packet_length});
    ll.add_location(Location {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(ll)
}

//process name resolution block
fn name_resolution(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let mut block = LocationList::new();
    block.add_location(Location {name: "name_resolution_block".to_owned(), offset, size: 4});
    block.add_location(Location {name: ".length".to_owned(), offset: offset + 4, size: 4});

    //parse records
    match parse_records(data, offset + 8, read_u16) {
        Err(s) => return Err(s),
        Ok(mut records) => {
            for loc in &mut records {
                let mut new_name = ".".to_owned();
                new_name.push_str(loc.name.as_str());
                loc.name = new_name;
            }
            block.extend(records)
        },
    }

    //parse options
    let last_offset = (&block).into_iter().last().unwrap().offset + 2;

    if block_length > (last_offset + 4) {
        match parse_options(data, last_offset, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                for loc in &mut options {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(loc.name.as_str());
                    loc.name = new_name;
                }
                block.extend(options)
            },
        }
    }

    block.add_location(Location {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

fn interface_statistics(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let mut block = LocationList::new();
    block.add_location(Location {name: "interface_statistics_block".to_owned(), offset, size: 4});
    block.add_location(Location {name: ".length".to_owned(), offset: offset + 4, size: 4});
    block.add_location(Location {name: ".interface_id".to_owned(), offset: offset + 8, size: 4});
    block.add_location(Location {name: ".time_stamp_h".to_owned(), offset: offset + 12, size: 4});
    block.add_location(Location {name: ".time_stamp_l".to_owned(), offset: offset + 16, size: 4});

    //parse options
    if block_length > 6*4 {
        match parse_options(data, offset + 20, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                for loc in &mut options {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(loc.name.as_str());
                    loc.name = new_name;
                }
                block.extend(options)
            },
        }
    }

    block.add_location(Location {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

//process enhanced packet block
fn enhanced_packet(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let captured_length = match read_u32(data, offset + 20) {
        Some(v) => v as usize,
        None => return Err("'captured_length' is out of data range!".to_owned()),
    };

    let mut block = LocationList::new();
    block.add_location(Location {name: "enhanced_packet_block".to_owned(), offset, size: 4});
    block.add_location(Location {name: ".length".to_owned(), offset: offset + 4, size: 4});
    block.add_location(Location {name: ".interface_id".to_owned(), offset: offset + 8, size: 4});
    block.add_location(Location {name: ".time_stamp_h".to_owned(), offset: offset + 12, size: 4});
    block.add_location(Location {name: ".time_stamp_l".to_owned(), offset: offset + 16, size: 4});
    block.add_location(Location {name: ".captured_length".to_owned(), offset: offset + 20, size: 4});
    block.add_location(Location {name: ".original_length".to_owned(), offset: offset + 24, size: 4});
    block.add_location(Location {name: ".data".to_owned(), offset: offset + 28, size: captured_length});

    let captured_aligned_length = (captured_length + 3) & !3;

    if block_length > (8*4 + captured_aligned_length) {
        match parse_options(data, offset + captured_aligned_length, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                for loc in &mut options {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(loc.name.as_str());
                    loc.name = new_name;
                }
                block.extend(options)
            },
        }
    }

    block.add_location(Location {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

//process decryption secrets block
fn decryption_secrets(data: &[u8], offset: usize, read_u16: &Read16u, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };

    let secrets_length = match read_u32(data, offset + 12) {
        Some(v) => v as usize,
        None => return Err("'captured_length' is out of data range!".to_owned()),
    };

    let mut block = LocationList::new();
    block.add_location(Location {name: "decryption_secrets_block".to_owned(), offset, size: 4});
    block.add_location(Location {name: ".length".to_owned(), offset: offset + 4, size: 4});
    block.add_location(Location {name: ".secrets_type".to_owned(), offset: offset + 8, size: 4});
    block.add_location(Location {name: ".secrets_length".to_owned(), offset: offset + 12, size: 4});
    block.add_location(Location {name: ".data".to_owned(), offset: offset + 16, size: secrets_length});

    let aligned_length = (secrets_length + 3) & !3;
    if block_length > (5*4 + aligned_length) {
        match parse_options(data, offset + aligned_length, read_u16) {
            Err(s) => return Err(s),
            Ok(mut options) => {
                for loc in &mut options {
                    let mut new_name = ".".to_owned();
                    new_name.push_str(loc.name.as_str());
                    loc.name = new_name;
                }
                block.extend(options)
            },
        }
    }

    block.add_location(Location {name: ".length".to_owned(), offset: offset + block_length - 4, size: 4});
    Ok(block)
}

fn unknown_block(data: &[u8], offset: usize, read_u32: &Read32u) -> Result<LocationList, String> {

    let block_length = match read_u32(data, offset + 4) {
        Some(v) => v as usize,
        None => return Err("'block_length' is out of data range!".to_owned()),
    };
    let mut ll = LocationList::new();
    ll.add_location(Location {name: "unknown_block".to_owned(), offset, size: block_length});
    Ok(ll)
}
