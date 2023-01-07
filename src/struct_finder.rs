//returns u32 from 4 bytes in little / high endian ordering
fn u32_from_bytes_unchecked(bytes: &[u8], little_endian: bool) -> u32 {
    if little_endian {
        (bytes[3] as u32) << 24 | (bytes[2] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[0] as u32
    } else {
        bytes[3] as u32 | (bytes[2] as u32) << 8 | (bytes[1] as u32) << 16 | (bytes[0] as u32) << 24
    }
}

//try to recognize 40 bytes DIB header. 
fn is_struct_dib(data: &[u8]) -> bool {
    
    if data.len() > 40 {

        //DIB header size
        if data[0] == 40 && data[1] == 0 && data[2] == 0 && data[3] == 0 {

            //number of planes must be 1
            if data[12] == 1 && data[13] == 0 {

                //bites per pixel 1,4,8,24
                return (data[14] == 1 || data[14] == 4 || data[14] == 8 || data[14] == 24 || data[14] == 32) && data[15] == 0;
            }
        }
    }

    false
}

//try to recognize BMP file ehader. And return its size
pub fn is_struct_bmp(data: &[u8]) -> Option<usize> {

    //we need to read at least 30 bytes
    if data.len() > 54 {

        //check 'BM' magic bytes
        if data[0] == 0x42 && data[1] == 0x4D {

            //check size
            let bmp_length = u32_from_bytes_unchecked(&data[2..6], true) as usize;
            if bmp_length < data.len() {

                //following 4 bytes are reserved and should be 0
                if data[6] == 0 && data[7] == 0 && data[8] == 0 && data[9] == 0 {
                    
                    //following 4 bytes are address of picture data. Should not be less then 0x36 and also not too much
                    let pic_offset = u32_from_bytes_unchecked(&data[10..14], true);
                    if pic_offset >= 0x36 && pic_offset <= 0xFFFF {
                        if is_struct_dib(&data[14..]) {
                            return Some(bmp_length);
                        }
                    }
                }
            }
        }
    }

    None
}

//try to recognize PNG file header
pub fn is_struct_png(data: &[u8]) -> Option<usize> {

    //TODO: better size checking
    //at least end of IHDR
    if data.len() > 32 {

        //PNG magic
        if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 && data[4] == 0x0D && data[5] == 0x0A && data[6] == 0x1A && data[7] == 0x0A {

            //next must be IHDR chunk. Is always 13 bytes long (big endian)
            if data[8] == 0 && data[9] == 0 && data[10] == 0 && data[11] == 0x0D && data[12] == 0x49 && data[13] == 0x48 && data[14] == 0x44 && data[15] == 0x52 {
                //TODO: png length -> iterate over all chunks until IEND chunk is found + its size
                return Some(0);
            }
        }
    }

    None
}

//try to recognize ICO/CUR file header
pub fn is_struct_ico(data: &[u8]) -> Option<usize> {

    //at least size of header + first image + size of png
    if data.len() > 54 {
        //ico starts with a lot of non uniqe bytes
        if data[0] == 0 && data[1] == 0 && (data[2] == 1 || data[2] == 2) && data[3] == 0 && data[5] == 0 && data[9] == 0 {

            let image_size = u32_from_bytes_unchecked(&data[14..18], true) as usize;
            if image_size < data.len() {

                let image_offset = u32_from_bytes_unchecked(&data[18..22] ,true) as usize;
                if image_offset >= 22 && image_offset < data.len() {

                    //need to check if it points to PNG or DIB struct
                    if is_struct_png(&data[image_offset..]).is_some() || is_struct_dib(&data[image_offset..]) {
                        return Some(0);
                    }
                }
            }
        }
    }

    None
}

//try to recognize GIF file header
pub fn is_struct_gif(data: &[u8]) -> Option<usize> {

    if data.len() > 100 {
        //check magic GIF89a
        if data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x38 && data[4] == 0x39 && data[5] == 0x61 {

            //find start of extension block. Should starts with '!'
            let offset = (1 << ((data[10] & 0x07) + 1) as usize) * 3 + 13;
            if data.len() > offset && data[offset] == 0x21 {
                return Some(0);
            }
        }
    }

    None
}

//try to recognize JPEG file header
pub fn is_struct_jpeg(data: &[u8]) -> Option<usize> {

    if data.len() > 50 {
        //check for FF D8 segment
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF { 

            //following with APP0 or APP1 segment with JFIF\x00 or Exif\x00 bytes
            if data[3] == 0xE0 {
                if data[6] == 0x4A && data[7] == 0x46 && data[8] == 0x49 && data[9] == 0x46 && data[10] == 0x00 {
                    return Some(0);
                }
            } else if data[3] == 0xE1 {
                if data[6] == 0x45 && data[7] == 0x78 && data[8] == 0x69 && data[9] == 0x66 && data[10] == 0x00 {
                    return Some(0);
                }
            }
        }
    }

    None
}
