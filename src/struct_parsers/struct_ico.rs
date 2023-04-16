use crate::signatures::is_signature;
use crate::struct_parsers::FieldDescription;

pub fn parse_ico_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "ico") {
        return Err("Invalid 'ico' signature!".to_owned());
    }

    let mut vec_image_data = Vec::<FieldDescription>::new();
    let mut vec_headers = Vec::<FieldDescription>::new();
    vec_headers.push(FieldDescription {name: "-- ICO --".to_owned(), offset: 0, size: 0});
    vec_headers.push(FieldDescription {name: "reserved".to_owned(), offset: 0, size: 2});
    vec_headers.push(FieldDescription {name: "type".to_owned(), offset: 2, size: 2});
    vec_headers.push(FieldDescription {name: "image_count".to_owned(), offset: 4, size: 2});

    //get number of images
    let icon_dir_entry_count = u16::from_le_bytes([data[4], data[5]]) as usize;

    //parse each ICONDIRENTRY. Size of ICONDIRENTRY is 16 bytes per entry
    for i in 0..icon_dir_entry_count {
        let icon_dir_entry_offset = 6 + 16*i;

        if data.len() < icon_dir_entry_offset + 16 {
            break;
        }

        vec_headers.push(FieldDescription {name: format!("-- ENTRY_{i} --"), offset: icon_dir_entry_offset, size: 0});
        vec_headers.push(FieldDescription {name: "width".to_owned(), offset: icon_dir_entry_offset, size: 1});
        vec_headers.push(FieldDescription {name: "height".to_owned(), offset: icon_dir_entry_offset + 1, size: 1});
        vec_headers.push(FieldDescription {name: "color_count".to_owned(), offset: icon_dir_entry_offset + 2, size: 1});
        vec_headers.push(FieldDescription {name: "reserved".to_owned(), offset: icon_dir_entry_offset + 3, size: 1});
        vec_headers.push(FieldDescription {name: "planes".to_owned(), offset: icon_dir_entry_offset + 4, size: 2});
        vec_headers.push(FieldDescription {name: "color_bits".to_owned(), offset: icon_dir_entry_offset + 6, size: 2});
        vec_headers.push(FieldDescription {name: "size".to_owned(), offset: icon_dir_entry_offset + 8, size: 4});
        vec_headers.push(FieldDescription {name: "offset".to_owned(), offset: icon_dir_entry_offset + 12, size: 4});

        vec_image_data.push(FieldDescription {
            name: format!("image_data"),
            offset: u32::from_le_bytes([data[icon_dir_entry_offset+12], data[icon_dir_entry_offset+13], data[icon_dir_entry_offset+14], data[icon_dir_entry_offset+15]]) as usize,
            size: u32::from_le_bytes([data[icon_dir_entry_offset+8], data[icon_dir_entry_offset+9], data[icon_dir_entry_offset+10], data[icon_dir_entry_offset+11]]) as usize
        });
    }
    vec_headers.append(&mut vec_image_data);
    Ok(vec_headers)
}
