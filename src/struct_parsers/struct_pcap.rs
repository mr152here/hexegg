use crate::signatures::is_signature;
use crate::struct_parsers::*;

pub fn parse_pcap_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "pcap") {
        return Err("Invalid 'PCAP' signature!".to_owned());
    }

    let mut header = vec![
        FieldDescription {name: "-- PCAP --".to_owned(), offset: 0, size: 0},
        FieldDescription {name: "block_type".to_owned(), offset: 0, size: 4},
        FieldDescription {name: "major".to_owned(), offset: 4, size: 2},
        FieldDescription {name: "minor".to_owned(), offset: 6, size: 2},
        FieldDescription {name: "reserved".to_owned(), offset: 8, size: 4},
        FieldDescription {name: "reserved".to_owned(), offset: 12, size: 4},
        FieldDescription {name: "snap_length".to_owned(), offset: 16, size: 4},
        FieldDescription {name: "link_type".to_owned(), offset: 20, size: 4},
    ];

    let little_endian = match read_le_u32(data, 0) {
        Some(v) if v == 0xA1B2C3D4 || v == 0xA1B23C4D => true,
        Some(v) if v == 0xD4C3B2A1 || v == 0x4D3CB2A1 => false,
        _ => return Err("Invalid endianess value!".to_owned()),
    };

    let read_u32 = if little_endian { read_le_u32 } else { read_be_u32 };

    //parse rest of the file as PCAP
    let mut last_offset = 24;
    while let Some(data_length) = read_u32(data, last_offset + 8) {
        let data_length = data_length as usize;

        header.push(FieldDescription {name: "packet_record".to_owned(), offset: last_offset, size: 0});
        header.push(FieldDescription {name: ".timestamp_s".to_owned(), offset: last_offset, size: 4});
        header.push(FieldDescription {name: ".timestamp_nu".to_owned(), offset: last_offset + 4, size: 4});
        header.push(FieldDescription {name: ".captured_length".to_owned(), offset: last_offset + 8, size: 4});
        header.push(FieldDescription {name: ".original_length".to_owned(), offset: last_offset + 12, size: 4});
        header.push(FieldDescription {name: ".data".to_owned(), offset: last_offset+ 16, size: data_length});

        last_offset += data_length + 16;
    }
    Ok(header)
}
