//returns u32 from 4 bytes in little / high endian ordering
fn u32_from_bytes_unchecked(bytes: &[u8], little_endian: bool) -> u32 {
    if little_endian {
        (bytes[3] as u32) << 24 | (bytes[2] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[0] as u32
    } else {
        bytes[3] as u32 | (bytes[2] as u32) << 8 | (bytes[1] as u32) << 16 | (bytes[0] as u32) << 24
    }
}

//try to check if it si BMP file ehader. And return its size
pub fn is_struct_bmp(data: &[u8]) -> Option<usize> {

    //we need to read at least 30 bytes
    if data.len() > 30 {

        //check 'BM' magic bytes
        if data[0] == 0x42 && data[1] == 0x4D {

            //check size
            let bmp_length = u32_from_bytes_unchecked(&data[2..6], true) as usize;
            if bmp_length >= data.len() {

                //following 4 bytes are reserved and should be 0
                if data[6] == 0 && data[7] == 0 && data[8] == 0 && data[9] == 0 {
                    
                    //following 4 bytes are address of picture data. Should not be less then 0x36 and also not too much
                    let pic_offset = u32_from_bytes_unchecked(&data[10..14], true);
                    if pic_offset >= 0x36 && pic_offset <= 0xFFFF {

                        //following 4 bytes are size of variants of BITMAP...HEADER. Most commonly 40 but not always. Not less then 12
                        if data[14] >= 12 && data[15] == 0 && data[16] == 0 && data[17] == 0 {

                            //number of planes, always 1
                            if data[26] == 1 && data[27] == 0 {

                                //bites per pixel
                                if (data[28] == 1 || data[28] == 4 || data[28] == 8 || data[28] == 24) && data[29] == 0 {
                                    return Some(bmp_length)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
