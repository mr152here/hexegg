use crate::signatures::is_signature;
use crate::location_list::{Location, LocationList};
use crate::struct_parsers::*;

pub fn parse_ico_struct(data: &[u8]) -> Result<LocationList, String> {

    if !is_signature(data, "ico") {
        return Err("Invalid 'ico' signature!".to_owned());
    }

    let mut header = LocationList::new();
    header.add_location(Location {name: "-- ICO --".to_owned(), offset: 0, size: 0});
    header.add_location(Location {name: "reserved".to_owned(), offset: 0, size: 2});
    header.add_location(Location {name: "type".to_owned(), offset: 2, size: 2});
    header.add_location(Location {name: "image_count".to_owned(), offset: 4, size: 2});

    //get number of images
    let icon_dir_entry_count = match read_le_u16(data, 4) {
        Some(v) => v as usize,
        None => return Err("ICO header is truncated!".to_owned()),
    };

    //parse each ICONDIRENTRY. Size of ICONDIRENTRY is 16 bytes per entry
    for i in 0..icon_dir_entry_count {
        let icon_dir_entry_offset = 6 + 16*i;

        header.add_location(Location {name: format!("-- ENTRY_{i} --"), offset: icon_dir_entry_offset, size: 0});
        header.add_location(Location {name: "width".to_owned(), offset: icon_dir_entry_offset, size: 1});
        header.add_location(Location {name: "height".to_owned(), offset: icon_dir_entry_offset + 1, size: 1});
        header.add_location(Location {name: "color_count".to_owned(), offset: icon_dir_entry_offset + 2, size: 1});
        header.add_location(Location {name: "reserved".to_owned(), offset: icon_dir_entry_offset + 3, size: 1});
        header.add_location(Location {name: "planes".to_owned(), offset: icon_dir_entry_offset + 4, size: 2});
        header.add_location(Location {name: "color_bits".to_owned(), offset: icon_dir_entry_offset + 6, size: 2});
        header.add_location(Location {name: "size".to_owned(), offset: icon_dir_entry_offset + 8, size: 4});
        header.add_location(Location {name: "offset".to_owned(), offset: icon_dir_entry_offset + 12, size: 4});

        let offset = match read_le_u32(data, icon_dir_entry_count + 12) {
            Some(v) => v as usize,
            None => return Err("ICON_DIR_ENTRY is truncated!".to_owned()),
        };

        let size = match read_le_u32(data, icon_dir_entry_count + 8) {
            Some(v) => v as usize,
            None => return Err("ICON_DIR_ENTRY is truncated!".to_owned()),
        };
        header.add_location(Location {name: "image_data".to_owned(), offset, size});
    }
    Ok(header)
}
