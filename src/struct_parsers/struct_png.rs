use crate::signatures::is_signature;
use crate::struct_parsers::*;

pub fn parse_png_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "png") {
        return Err("Invalid 'png' signature!".to_owned());
    }

    let mut vec_headers = Vec::<FieldDescription>::new();
    vec_headers.push(FieldDescription {name: "-- PNG --".to_owned(), offset: 0, size: 0});
    vec_headers.push(FieldDescription {name: "magic".to_owned(), offset: 0, size: 8});

    //iterate over all chunks
    let mut offset = 8;
    let mut iend_chunk = false;

    while !iend_chunk && data.len() >= offset + 8 {

        let chunk_size = match read_be_u32(data, offset) {
            Some(v) => v as usize,
            None => return Err("PNG chunk is truncated!".to_owned()),
        };

        vec_headers.push(FieldDescription {name: "chunk_size".to_owned(), offset, size: 4});

        iend_chunk = match read_be_u32(data, offset + 4) {
            Some(v) => v == 0x49454E44,
            None => return Err("PNG chunk is truncated!".to_owned()),
        };

        vec_headers.push(FieldDescription {name: "chunk_type".to_owned(), offset: offset + 4, size: 4});

        if chunk_size > 0 {
            vec_headers.push(FieldDescription {name: "chunk_data".to_owned(), offset: offset + 8, size: chunk_size});
            offset += chunk_size;
        }

        vec_headers.push(FieldDescription {name: "chunk_crc".to_owned(), offset: offset + 8, size: 4});
        offset += 12;
    }
    Ok(vec_headers)
}
