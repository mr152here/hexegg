use crate::signatures::is_signature;
use crate::location_list::{Location, LocationList};
use crate::struct_parsers::*;

pub fn parse_png_struct(data: &[u8]) -> Result<LocationList, String> {

    if !is_signature(data, "png") {
        return Err("Invalid 'PNG' signature!".to_owned());
    }

    let mut header = LocationList::new();
    header.add_location(Location {name: "-- PNG --".to_owned(), offset: 0, size: 0});
    header.add_location(Location {name: "magic".to_owned(), offset: 0, size: 8});

    //iterate over all chunks
    let mut offset = 8;
    let mut iend_chunk = false;

    while !iend_chunk && data.len() >= offset + 8 {

        let chunk_size = match read_be_u32(data, offset) {
            Some(v) => v as usize,
            None => return Err("PNG chunk is truncated!".to_owned()),
        };

        header.add_location(Location {name: "chunk".to_owned(), offset, size: 0});
        header.add_location(Location {name: ".chunk_size".to_owned(), offset, size: 4});

        iend_chunk = match read_be_u32(data, offset + 4) {
            Some(v) => v == 0x49454E44,
            None => return Err("PNG chunk is truncated!".to_owned()),
        };

        header.add_location(Location {name: ".chunk_type".to_owned(), offset: offset + 4, size: 4});

        if chunk_size > 0 {
            header.add_location(Location {name: ".chunk_data".to_owned(), offset: offset + 8, size: chunk_size});
            offset += chunk_size;
        }

        header.add_location(Location {name: ".chunk_crc".to_owned(), offset: offset + 8, size: 4});
        offset += 12;
    }
    Ok(header)
}
