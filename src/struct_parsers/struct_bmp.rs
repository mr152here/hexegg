use crate::signatures::is_signature;
use crate::struct_parsers::FieldDescription;

pub fn parse_bmp_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "bmp") {
        return Err("Invalid 'bmp' signature!".to_owned());

    } else if data.len() < 36 {
        return Err("Too small for 'bmp' header!".to_owned());
    }

    //BITMAPFILEHEADER
    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "-- bmp --".to_owned(), offset: 0, size: 0});
    header.push(FieldDescription {name: "magic".to_owned(), offset: 0, size: 2});
    header.push(FieldDescription {name: "file_size".to_owned(), offset: 2, size: 4});
    header.push(FieldDescription {name: "reserved".to_owned(), offset: 6, size: 2});
    header.push(FieldDescription {name: "reserved".to_owned(), offset: 8, size: 2});
    header.push(FieldDescription {name: "image_offset".to_owned(), offset: 10, size: 4});

    match parse_dib_struct(&data[14..]) {
        Ok(mut vec_dib) => {
            vec_dib.iter_mut().for_each(|fd| fd.offset += 14 );
            header.append(&mut vec_dib);
        },
        Err(s) => return Err(s),
    }

    let image_offset = u32::from_le_bytes(data[10..14].try_into().unwrap()) as usize;
    let image_size = dib_image_size(&data[14..]).unwrap_or((u32::from_le_bytes(data[2..6].try_into().unwrap()) as usize) - image_offset);

    header.push(FieldDescription {name: "image_data".to_owned(), offset: image_offset, size: image_size});
    Ok(header)
}

pub fn parse_dib_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if data.len() < 12 {
        return Err("Too small for DIB header!".to_owned());
    }

    let mut header = Vec::<FieldDescription>::new();
    let dib_size = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;

    if dib_size == 12 {
        //BITMAPCOREHEADER
        header.push(FieldDescription {name: "-- dib --".to_owned(), offset: 0, size: 0});
        header.push(FieldDescription {name: "size".to_owned(), offset: 0, size: 4});
        header.push(FieldDescription {name: "width".to_owned(), offset: 4, size: 2});
        header.push(FieldDescription {name: "height".to_owned(), offset: 6, size: 2});
        header.push(FieldDescription {name: "planes".to_owned(), offset: 8, size: 2});
        header.push(FieldDescription {name: "bit_counts".to_owned(), offset: 10, size: 2});

    } else if data.len() >= 40 && (dib_size == 40 || dib_size == 108 || dib_size == 124) {
        //BITMAPINFOHEADER
        header.push(FieldDescription {name: "-- dib --".to_owned(), offset: 0, size: 0});
        header.push(FieldDescription {name: "size".to_owned(), offset: 0, size: 4});
        header.push(FieldDescription {name: "width".to_owned(), offset: 4, size: 4});
        header.push(FieldDescription {name: "height".to_owned(), offset: 8, size: 4});
        header.push(FieldDescription {name: "planes".to_owned(), offset: 12, size: 2});
        header.push(FieldDescription {name: "bits".to_owned(), offset: 14, size: 2});
        header.push(FieldDescription {name: "compression".to_owned(), offset: 16, size: 4});
        header.push(FieldDescription {name: "image_size".to_owned(), offset: 20, size: 4});
        header.push(FieldDescription {name: "xpix_pm".to_owned(), offset: 24, size: 4});
        header.push(FieldDescription {name: "ypix_pm".to_owned(), offset: 28, size: 4});
        header.push(FieldDescription {name: "colors".to_owned(), offset: 32, size: 4});
        header.push(FieldDescription {name: "imp_colors".to_owned(), offset: 36, size: 4});

        if data.len() >= 108 && (dib_size == 108 || dib_size == 124) {
            //BITMAPV4HEADER
            header.push(FieldDescription {name: "mask_red".to_owned(), offset: 40, size: 4});
            header.push(FieldDescription {name: "mask_green".to_owned(), offset: 44, size: 4});
            header.push(FieldDescription {name: "mask_blue".to_owned(), offset: 48, size: 4});
            header.push(FieldDescription {name: "mask_alpha".to_owned(), offset: 52, size: 4});
            header.push(FieldDescription {name: "cs_type".to_owned(), offset: 56, size: 4});
            header.push(FieldDescription {name: "red_x".to_owned(), offset: 60, size: 4});
            header.push(FieldDescription {name: "red_y".to_owned(), offset: 64, size: 4});
            header.push(FieldDescription {name: "red_z".to_owned(), offset: 68, size: 4});
            header.push(FieldDescription {name: "green_x".to_owned(), offset: 72, size: 4});
            header.push(FieldDescription {name: "green_y".to_owned(), offset: 76, size: 4});
            header.push(FieldDescription {name: "green_z".to_owned(), offset: 80, size: 4});
            header.push(FieldDescription {name: "blue_x".to_owned(), offset: 84, size: 4});
            header.push(FieldDescription {name: "blue_y".to_owned(), offset: 88, size: 4});
            header.push(FieldDescription {name: "blue_z".to_owned(), offset: 92, size: 4});
            header.push(FieldDescription {name: "gamma_red".to_owned(), offset: 96, size: 4});
            header.push(FieldDescription {name: "gamma_green".to_owned(), offset: 100, size: 4});
            header.push(FieldDescription {name: "gamma_blue".to_owned(), offset: 104, size: 4});

            if data.len() >= 124 && dib_size == 124{
                //BITMAPV5HEADER
                header.push(FieldDescription {name: "intent".to_owned(), offset: 108, size: 4});
                header.push(FieldDescription {name: "profile_offset".to_owned(), offset: 112, size: 4});
                header.push(FieldDescription {name: "profile_size".to_owned(), offset: 116, size: 4});
                header.push(FieldDescription {name: "reserved".to_owned(), offset: 120, size: 4});

                let prof_offset = u32::from_le_bytes(data[112..116].try_into().unwrap()) as usize;
                let prof_size = u32::from_le_bytes(data[116..120].try_into().unwrap()) as usize;
                header.push(FieldDescription {name: "profile_data".to_owned(), offset: prof_offset, size: prof_size});
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
    if data.len() >= 40 {
        let dib_size = u32::from_le_bytes(data[0..4].try_into().unwrap());

        if dib_size >= 40 {
            return Some(u32::from_le_bytes(data[20..24].try_into().unwrap()) as usize);
        }
    }
    None
}
