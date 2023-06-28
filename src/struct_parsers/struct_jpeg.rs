use crate::signatures::is_signature;
use crate::location_list::{Location, LocationList};
use crate::struct_parsers::*;

type Read16u = dyn Fn(&[u8], usize) -> Option<u16>;

pub fn parse_jpeg_struct(data: &[u8]) -> Result<LocationList, String> {

    if !is_signature(data, "jpeg") {
        return Err("Invalid 'JPEG' signature!".to_owned());
    }

    let mut end_of_image = false;
    let mut offset: usize = 0;
    let mut header = LocationList::new();
    header.add_location(Location {name: "-- JPEG --".to_owned(), offset: 0, size: 0});

    //loop through all segments
    while let Some(byte) = read_u8(data, offset) {
        if byte == 0xFF {
            let segment_ll = match read_u8(data, offset + 1) {
                Some(segment_name) => match segment_name {
                    0xC0 => parse_sof0(data, &mut offset, &read_be_u16),
                    0xC2 => parse_sof2(data, &mut offset, &read_be_u16),
                    0xC4 => parse_dht(data, &mut offset, &read_be_u16),
                    0xD0..=0xD7 => parse_rstn(data, &mut offset),
                    0xD8 => parse_soi(&mut offset),
                    0xD9 => { end_of_image = true; parse_eoi(&mut offset) },
                    0xDB => parse_dqt(data, &mut offset, &read_be_u16),
                    0xDA => parse_sos(data, &mut offset, &read_be_u16),
                    0xDD => parse_dri(&mut offset),
                    0xE0..=0xE2 => parse_appn(data, &mut offset, &read_be_u16),
                    0xFE => parse_com(data, &mut offset, &read_be_u16),
                    _ => {offset += 1; continue;},
                },
                None => break,
            };

            match segment_ll {
                Ok(sll) => header.extend(sll),
                Err(s) => return Err(s),
            }

            if end_of_image {
                break;
            }
        } else {
            offset += 1;
        }
    }

    Ok(header)
}

fn parse_sof0(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_SOF0".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}

fn parse_sof2(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_SOF2".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}

fn parse_dht(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_DHT".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}

fn parse_rstn(data: &[u8], offset: &mut usize) -> Result<LocationList, String> {

    let rst_num = match data.get(1) {
        Some(v) => v & 0x0F,
        None => return Err("'RSTn' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: format!("segment_RST{}", rst_num), offset: *offset, size: 2});
    *offset += 2;

    Ok(ll)
}

fn parse_soi(offset: &mut usize) -> Result<LocationList, String> {

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_SOI".to_owned(), offset: *offset, size: 2});
    *offset += 2;

    Ok(ll)
}

fn parse_eoi(offset: &mut usize) -> Result<LocationList, String> {

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_EOI".to_owned(), offset: *offset, size: 2});
    *offset += 2;

    Ok(ll)
}

fn parse_dqt(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_DQT".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}

fn parse_sos(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_SOS".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}

fn parse_dri(offset: &mut usize) -> Result<LocationList, String> {

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_DQT".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 2, size: 4});
    *offset += 6;

    Ok(ll)
}

fn parse_appn(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let app_num = match data.get(*offset + 1) {
        Some(v) => v & 0x0F,
        None => return Err("'APPn' is out of data range!".to_owned()),
    };

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: format!("segment_APP{}", app_num), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}

fn parse_com(data: &[u8], offset: &mut usize, read_u16: &Read16u) -> Result<LocationList, String> {

    let segment_size = match read_u16(data, *offset + 2) {
        Some(v) => v as usize,
        None => return Err("'segment_size' is out of data range!".to_owned()),
    };

    let mut ll = LocationList::new();
    ll.add_location(Location {name: "segment_COM".to_owned(), offset: *offset, size: 2});
    ll.add_location(Location {name: ".size".to_owned(), offset: *offset + 2, size: 2});
    ll.add_location(Location {name: ".data".to_owned(), offset: *offset + 4, size: segment_size - 2});
    *offset += segment_size + 2;

    Ok(ll)
}
