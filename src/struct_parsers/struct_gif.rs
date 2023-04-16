use crate::signatures::is_signature;
use crate::struct_parsers::FieldDescription;

// https://www.w3.org/Graphics/GIF/spec-gif89a.txt
pub fn parse_gif_struct(data: &[u8]) -> Result<Vec<FieldDescription>, String> {

    if !is_signature(data, "gif") {
        return Err("Invalid 'gif' signature!".to_owned());
    }

    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "-- GIF --".to_owned(), offset: 0, size: 0});
    header.push(FieldDescription {name: "magic".to_owned(), offset: 0, size: 6});
    header.push(FieldDescription {name: "width".to_owned(), offset: 6, size: 2});
    header.push(FieldDescription {name: "height".to_owned(), offset: 8, size: 2});
    header.push(FieldDescription {name: "flags".to_owned(), offset: 10, size: 1});
    header.push(FieldDescription {name: "bg_color_idx".to_owned(), offset: 11, size: 1});
    header.push(FieldDescription {name: "pixel_aspect_ratio".to_owned(), offset: 12, size: 1});
    let mut last_offset = 13 as usize;

    // if global color table flag from flags is set then global color table follows with size 3 * 2 ^ (global color table size + 1)
    let flags = match data.get(10) {
        Some(&f) => f,
        None => return Err("File seems to be corrupted!".to_owned()),
    };

    if (flags & 0x80) != 0 {
        let global_color_table_size = 3 * (2 << (flags & 0x07));
        header.push(FieldDescription {name: "gl_color_table".to_owned(), offset: last_offset, size: global_color_table_size});
        last_offset += global_color_table_size;
    }

    loop {
        let block_label = match data.get(last_offset) {
            Some(bl) => *bl,
            None => break,
        };

        //gif trialer / end
        if block_label == 0x3B {
            header.push(FieldDescription {name: "gif_trailer".to_owned(), offset: last_offset, size: 1});
            break;

        //process image descriptor
        } else if block_label == 0x2C {
            header.push(FieldDescription {name: "-- IMAGE DESCRIPTOR --".to_owned(), offset: last_offset, size: 1});
            header.push(FieldDescription {name: "left_position".to_owned(), offset: last_offset+1, size: 2});
            header.push(FieldDescription {name: "top_position".to_owned(), offset: last_offset+3, size: 2});
            header.push(FieldDescription {name: "width".to_owned(), offset: last_offset+5, size: 2});
            header.push(FieldDescription {name: "height".to_owned(), offset: last_offset+7, size: 2});
            header.push(FieldDescription {name: "flags".to_owned(), offset: last_offset+9, size: 1});
            last_offset += 10;

            let flags = match data.get(last_offset-1) {
                Some(&f) => f,
                // None => return Err("File seems to be corrupted!".to_owned()),
                None => break,
            };

            if (flags & 0x80) != 0 {
                let local_color_table_size = 3 * (2 << (flags & 0x07));
                header.push(FieldDescription {name: "loc_color_table".to_owned(), offset: last_offset, size: local_color_table_size});
                last_offset += local_color_table_size;
            }

            //parse image blocks
            header.push(FieldDescription {name: "LZW_code_size".to_owned(), offset: last_offset, size: 1});
            last_offset += 1;
            let block_start = last_offset;

            while let Some(&sub_block_size) = data.get(last_offset) {

                if sub_block_size == 0 {
                    header.push(FieldDescription {name: "sub_blocks".to_owned(), offset: block_start, size: last_offset});
                    last_offset += 1;
                    break;
                }

                last_offset += 1 + sub_block_size as usize;
            }

        //parse extendsion blocks
        } else if block_label == 0x21 {
            last_offset += 1;
            let label = match data.get(last_offset) {
                Some(lb) => *lb,
                // None => return Err("File seems to be corrupted!".to_owned()),
                None => break,
            };

            last_offset += 1;
            let mut block_size = 0;
            let extension_header = match label {
                0x01 => {
                    header.push(FieldDescription {name: "-- PLAIN TEXT EXTENSION --".to_owned(), offset: last_offset-2, size: 2});
                    parse_plain_text_extension(&data[last_offset..], &mut block_size)
                },
                0xF9 => {
                    header.push(FieldDescription {name: "-- GRAPHICS CONTROL EXTENSION --".to_owned(), offset: last_offset-2, size: 2});
                    parse_graphic_control_extension(&data[last_offset..], &mut block_size)
                },
                0xFE => {
                    header.push(FieldDescription {name: "-- COMMENT EXTENSION --".to_owned(), offset: last_offset-2, size: 2});
                    parse_comment_extension(&data[last_offset..], &mut block_size)
                },
                0xFF => {
                    header.push(FieldDescription {name: "-- APP EXTENSION --".to_owned(), offset: last_offset-2, size: 2});
                    parse_application_extension(&data[last_offset..], &mut block_size)
                },
                lb => return Err(format!("Unknown/unsupported extension label {:0X}", lb)),
            };

            extension_header.into_iter().for_each(|mut fd| {
                fd.offset += last_offset;
                header.push(fd);
            });
            last_offset += block_size;

        } else {
            return Err(format!("Unknown/unsupported block label {:0X}", block_label));
        }
    }

    Ok(header)
}

fn parse_plain_text_extension(data: &[u8], size: &mut usize) -> Vec<FieldDescription> {
    let mut last_offset = 13;
    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "block_size".to_owned(), offset: 0, size: 1});
    header.push(FieldDescription {name: "left".to_owned(), offset: 1, size: 2});
    header.push(FieldDescription {name: "top".to_owned(), offset: 3, size: 2});
    header.push(FieldDescription {name: "width".to_owned(), offset: 5, size: 2});
    header.push(FieldDescription {name: "height".to_owned(), offset: 7, size: 2});
    header.push(FieldDescription {name: "cell_width".to_owned(), offset: 9, size: 1});
    header.push(FieldDescription {name: "cell_height".to_owned(), offset: 10, size: 1});
    header.push(FieldDescription {name: "fg_color_idx".to_owned(), offset: 11, size: 1});
    header.push(FieldDescription {name: "bg_color_idx".to_owned(), offset: 12, size: 1});

    //loop over all subblocks until 0 sized block (terminator) is found
    while let Some(&sub_block_size) = data.get(last_offset) {

        if sub_block_size == 0 {
            header.push(FieldDescription {name: "sub_blocks".to_owned(), offset: 13, size: last_offset});
            last_offset += 1;
            break;
        }

        last_offset += 1 + sub_block_size as usize;
    }

    *size = last_offset;
    header
}

fn parse_graphic_control_extension(data: &[u8], size: &mut usize) -> Vec<FieldDescription> {
    let last_offset = 6;
    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "block_size".to_owned(), offset: 0, size: 1});
    header.push(FieldDescription {name: "flags".to_owned(), offset: 1, size: 1});
    header.push(FieldDescription {name: "delay_time".to_owned(), offset: 2, size: 2});
    header.push(FieldDescription {name: "tr_color_idx".to_owned(), offset: 4, size: 1});
    header.push(FieldDescription {name: "block_terminator".to_owned(), offset: 5, size: 1});
    // TODO: possible bug? transparency color index may be present only if coresponding flag is set
    *size = last_offset;
    header
}


fn parse_comment_extension(data: &[u8], size: &mut usize) -> Vec<FieldDescription> {
    let mut last_offset = 1;
    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "block_size".to_owned(), offset: 0, size: 1});

    //loop over all subblocks until 0 sized block (terminator) is found
    while let Some(&sub_block_size) = data.get(last_offset) {

        if sub_block_size == 0 {
            header.push(FieldDescription {name: "sub_blocks".to_owned(), offset: 1, size: last_offset});
            last_offset += 1;
            break;
        }

        last_offset += 1 + sub_block_size as usize;
    }

    *size = last_offset;
    header
}

fn parse_application_extension(data: &[u8], size: &mut usize) -> Vec<FieldDescription> {
    let mut last_offset = 12;
    let mut header = Vec::<FieldDescription>::new();
    header.push(FieldDescription {name: "block_size".to_owned(), offset: 0, size: 1});
    header.push(FieldDescription {name: "app_identifier".to_owned(), offset: 1, size: 8});
    header.push(FieldDescription {name: "app_auth_code".to_owned(), offset: 9, size: 3});

    //loop over all subblocks until 0 sized block (terminator) is found
    while let Some(&sub_block_size) = data.get(last_offset) {

        if sub_block_size == 0 {
            header.push(FieldDescription {name: "sub_blocks".to_owned(), offset: 12, size: last_offset});
            last_offset += 1;
            break;
        }

        last_offset += 1 + sub_block_size as usize;
    }

    *size = last_offset;
    header
}
