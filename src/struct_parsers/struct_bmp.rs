use crate::signatures::is_signature;
use crate::location_list::{Location, LocationList};
use crate::struct_parsers::*;

pub fn parse_bmp_struct(data: &[u8]) -> Result<LocationList, String> {

    if !is_signature(data, "bmp") {
        return Err("Invalid 'bmp' signature!".to_owned());

    } else if data.len() < 36 {
        return Err("Too small for 'bmp' header!".to_owned());
    }

    //BITMAPFILEHEADER
    let mut header = LocationList::new();
    header.add_location(Location {name: "-- BMP --".to_owned(), offset: 0, size: 0});
    header.add_location(Location {name: "magic".to_owned(), offset: 0, size: 2});
    header.add_location(Location {name: "file_size".to_owned(), offset: 2, size: 4});
    header.add_location(Location {name: "reserved".to_owned(), offset: 6, size: 4});
    header.add_location(Location {name: "image_offset".to_owned(), offset: 10, size: 4});

    match parse_dib_struct(&data[14..]) {
        Ok(dib) => {
            dib.into_iter().for_each(|mut loc| {
                loc.offset += 14;
                header.add_location(loc);
            });
        },
        Err(s) => return Err(s),
    }

    let image_offset = match read_le_u32(data, 10) {
        Some(v) => v as usize,
        None => return Err("BMP header is truncated!".to_owned()),
    };

    let file_size = match read_le_u32(data, 2) {
        Some(v) => v as usize,
        None => return Err("BMP header is truncated!".to_owned()),
    };

    let image_size = dib_image_size(&data[14..]).unwrap_or(file_size - image_offset);

    header.add_location(Location {name: "image_data".to_owned(), offset: image_offset, size: image_size});
    Ok(header)
}

pub fn parse_dib_struct(data: &[u8]) -> Result<LocationList, String> {

    let dib_size = match read_le_u32(data, 0) {
        Some(v) => v as usize,
        None => return Err("DIB header is truncated!".to_owned()),
    };

    let mut header = LocationList::new();

    if dib_size == 12 {
        //BITMAPCOREHEADER
        header.add_location(Location {name: "-- DIB --".to_owned(), offset: 0, size: 0});
        header.add_location(Location {name: "size".to_owned(), offset: 0, size: 4});
        header.add_location(Location {name: "width".to_owned(), offset: 4, size: 2});
        header.add_location(Location {name: "height".to_owned(), offset: 6, size: 2});
        header.add_location(Location {name: "planes".to_owned(), offset: 8, size: 2});
        header.add_location(Location {name: "bit_counts".to_owned(), offset: 10, size: 2});

    } else if dib_size == 40 || dib_size == 108 || dib_size == 124 {
        //BITMAPINFOHEADER
        header.add_location(Location {name: "-- DIB --".to_owned(), offset: 0, size: 0});
        header.add_location(Location {name: "size".to_owned(), offset: 0, size: 4});
        header.add_location(Location {name: "width".to_owned(), offset: 4, size: 4});
        header.add_location(Location {name: "height".to_owned(), offset: 8, size: 4});
        header.add_location(Location {name: "planes".to_owned(), offset: 12, size: 2});
        header.add_location(Location {name: "bits".to_owned(), offset: 14, size: 2});
        header.add_location(Location {name: "compression".to_owned(), offset: 16, size: 4});
        header.add_location(Location {name: "image_size".to_owned(), offset: 20, size: 4});
        header.add_location(Location {name: "xpix_pm".to_owned(), offset: 24, size: 4});
        header.add_location(Location {name: "ypix_pm".to_owned(), offset: 28, size: 4});
        header.add_location(Location {name: "colors".to_owned(), offset: 32, size: 4});
        header.add_location(Location {name: "imp_colors".to_owned(), offset: 36, size: 4});

        if dib_size == 108 || dib_size == 124 {
            //BITMAPV4HEADER
            header.add_location(Location {name: "mask_red".to_owned(), offset: 40, size: 4});
            header.add_location(Location {name: "mask_green".to_owned(), offset: 44, size: 4});
            header.add_location(Location {name: "mask_blue".to_owned(), offset: 48, size: 4});
            header.add_location(Location {name: "mask_alpha".to_owned(), offset: 52, size: 4});
            header.add_location(Location {name: "cs_type".to_owned(), offset: 56, size: 4});
            header.add_location(Location {name: "red_x".to_owned(), offset: 60, size: 4});
            header.add_location(Location {name: "red_y".to_owned(), offset: 64, size: 4});
            header.add_location(Location {name: "red_z".to_owned(), offset: 68, size: 4});
            header.add_location(Location {name: "green_x".to_owned(), offset: 72, size: 4});
            header.add_location(Location {name: "green_y".to_owned(), offset: 76, size: 4});
            header.add_location(Location {name: "green_z".to_owned(), offset: 80, size: 4});
            header.add_location(Location {name: "blue_x".to_owned(), offset: 84, size: 4});
            header.add_location(Location {name: "blue_y".to_owned(), offset: 88, size: 4});
            header.add_location(Location {name: "blue_z".to_owned(), offset: 92, size: 4});
            header.add_location(Location {name: "gamma_red".to_owned(), offset: 96, size: 4});
            header.add_location(Location {name: "gamma_green".to_owned(), offset: 100, size: 4});
            header.add_location(Location {name: "gamma_blue".to_owned(), offset: 104, size: 4});

            if dib_size == 124 {
                //BITMAPV5HEADER
                header.add_location(Location {name: "intent".to_owned(), offset: 108, size: 4});
                header.add_location(Location {name: "profile_offset".to_owned(), offset: 112, size: 4});
                header.add_location(Location {name: "profile_size".to_owned(), offset: 116, size: 4});
                header.add_location(Location {name: "reserved".to_owned(), offset: 120, size: 4});

                let prof_offset = match read_le_u32(data, 112) {
                    Some(v) => v as usize,
                    None => return Err("DIB header is truncated!".to_owned()),
                };

                let prof_size = match read_le_u32(data, 116) {
                    Some(v) => v as usize,
                    None => return Err("DIB header is truncated!".to_owned()),
                };

                header.add_location(Location {name: "profile_data".to_owned(), offset: prof_offset, size: prof_size});
            }
        }
    }

    match header.is_empty() {
        true => Err("unknown DIB header format!".to_string()),
        false => Ok(header),
    }
}

//returns image size from DIB header.
pub fn dib_image_size(data: &[u8]) -> Option<usize> {
    if let Some(dib_size) = read_le_u32(data, 0) {
        if dib_size >= 40 {
            return read_le_u32(data, 20).map(|v| v as usize);
        }
    }
    None
}
