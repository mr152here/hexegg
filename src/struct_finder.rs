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






