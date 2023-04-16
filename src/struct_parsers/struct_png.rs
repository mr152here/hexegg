use crate::signatures::is_signature;
use crate::struct_parsers::FieldDescription;

pub fn parse_png_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "png") {
        return Err("Invalid 'png' signature!".to_owned());
    }

    let mut vec_headers = Vec::<FieldDescription>::new();
    let mut offset = 8;
    let mut iend_chunk = false;

    vec_headers.push(FieldDescription {name: "-- PNG --".to_owned(), offset: 0, size: 0});
    vec_headers.push(FieldDescription {name: "magic".to_owned(), offset: 0, size: 8});

    //iterate over all chunks
    while !iend_chunk && data.len() >= offset + 8 {

        let chunk_size = u32::from_be_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        vec_headers.push(FieldDescription {name: "chunk_size".to_owned(), offset, size: 4});
        offset += 4;

        iend_chunk = u32::from_be_bytes(data[offset..offset+4].try_into().unwrap()) == 0x49454E44;
        vec_headers.push(FieldDescription {name: "chunk_type".to_owned(), offset, size: 4});
        offset += 4;

        if chunk_size > 0 {
            vec_headers.push(FieldDescription {name: "chunk_data".to_owned(), offset, size: chunk_size});
            offset += chunk_size;
        }

        vec_headers.push(FieldDescription {name: "chunk_crc".to_owned(), offset, size: 4});
        offset += 4;
    }
    Ok(vec_headers)
}
