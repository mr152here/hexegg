use crate::signatures::is_signature;
use crate::struct_parsers::*;

pub fn parse_ico_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "ico") {
        return Err("Invalid 'ico' signature!".to_owned());
    }

    let mut vec_image_data = Vec::<FieldDescription>::new();
    let mut vec_headers = vec![
        FieldDescription {name: "-- ICO --".to_owned(), offset: 0, size: 0},
        FieldDescription {name: "reserved".to_owned(), offset: 0, size: 2},
        FieldDescription {name: "type".to_owned(), offset: 2, size: 2},
        FieldDescription {name: "image_count".to_owned(), offset: 4, size: 2}
    ];

    //get number of images
    let icon_dir_entry_count = match read_le_u16(&data, 4) {
        Some(v) => v as usize,
        None => return Err("ICO header is truncated!".to_owned()),
    };

    //parse each ICONDIRENTRY. Size of ICONDIRENTRY is 16 bytes per entry
    for i in 0..icon_dir_entry_count {
        let icon_dir_entry_offset = 6 + 16*i;

        vec_headers.push(FieldDescription {name: format!("-- ENTRY_{i} --"), offset: icon_dir_entry_offset, size: 0});
        vec_headers.push(FieldDescription {name: "width".to_owned(), offset: icon_dir_entry_offset, size: 1});
        vec_headers.push(FieldDescription {name: "height".to_owned(), offset: icon_dir_entry_offset + 1, size: 1});
        vec_headers.push(FieldDescription {name: "color_count".to_owned(), offset: icon_dir_entry_offset + 2, size: 1});
        vec_headers.push(FieldDescription {name: "reserved".to_owned(), offset: icon_dir_entry_offset + 3, size: 1});
        vec_headers.push(FieldDescription {name: "planes".to_owned(), offset: icon_dir_entry_offset + 4, size: 2});
        vec_headers.push(FieldDescription {name: "color_bits".to_owned(), offset: icon_dir_entry_offset + 6, size: 2});
        vec_headers.push(FieldDescription {name: "size".to_owned(), offset: icon_dir_entry_offset + 8, size: 4});
        vec_headers.push(FieldDescription {name: "offset".to_owned(), offset: icon_dir_entry_offset + 12, size: 4});

        let offset = match read_le_u32(&data, icon_dir_entry_count + 12) {
            Some(v) => v as usize,
            None => return Err("ICON_DIR_ENTRY is truncated!".to_owned()),
        };

        let size = match read_le_u32(&data, icon_dir_entry_count + 8) {
            Some(v) => v as usize,
            None => return Err("ICON_DIR_ENTRY is truncated!".to_owned()),
        };

        vec_image_data.push(FieldDescription {name: "image_data".to_owned(), offset, size});
    }
    vec_headers.append(&mut vec_image_data);
    Ok(vec_headers)
}
