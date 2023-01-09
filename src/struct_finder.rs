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

//try to recognize ZIP header
pub fn is_struct_zip(data: &[u8]) -> Option<usize> {

    if data.len() > 8 {

        //check for PK magic
        if data[0] == 0x50 && data[1] == 0x4B {
            //check for various block types
            let d2 = data[2];
            let d3 = data[3];
            if (d2 == 3 && d3 == 4) || (d2 == 6 && d3 == 8) || (d2 == 1 && d3 == 2) || (d2 == 6 && d3 == 6) || (d2 == 6 && d3 == 7)  || (d2 == 5 && d3 == 6) || (d2 == 5 && d3 == 5) { 
                return Some(0);
            }
        }
    }

    None
}

//try to recognize RAR header
pub fn is_struct_rar(data: &[u8]) -> Option<usize> {

    if data.len() > 8 {

        //check for Rar! magic
        if data[0] == 0x52 && data[1] == 0x61 && data[2] == 0x72 && data[3] == 0x21 && data[4] == 0x1A && data[5] == 0x07 && (data[6] == 0x01 || data[6] == 0x00) { 
            return Some(0);
        }
    }

    None
}

//try to recognize 7z header
pub fn is_struct_7z(data: &[u8]) -> Option<usize> {

    if data.len() > 32 {

        //check for 7z magic
        if data[0] == 0x37 && data[1] == 0x7A && data[2] == 0xBC && data[3] == 0xAF && data[4] == 0x27 && data[5] == 0x1C { 
            return Some(0);
        }
    }

    None
}

//try to recognize xz header
pub fn is_struct_xz(data: &[u8]) -> Option<usize> {

    if data.len() > 12 {

        //check for xz magic
        if data[0] == 0xFD && data[1] == 0x37 && data[2] == 0x7A && data[3] == 0x58 && data[4] == 0x5A && data[5] == 0 && data[6] == 0 && data[7] <= 0x0F { 
            return Some(0);
        }
    }

    None
}

//try to recognize bzip2 header
pub fn is_struct_bzip2(data: &[u8]) -> Option<usize> {

    if data.len() > 20 {

        //check for bzip2 magic
        if data[0] == 0x42 && data[1] == 0x5A && data[2] == 0x68 && data[3] >= 0x31 && data[3] <= 0x39 && data[4] == 0x31 && data[5] == 0x41 && data[6] == 0x59 && data[7] == 0x26 && data[8] == 0x53 && data[9] == 0x59 { 
            return Some(0);
        }
    }

    None
}

//try to recognize gz header
pub fn is_struct_gzip(data: &[u8]) -> Option<usize> {

    if data.len() > 100 {

        //check for gzip magic
        if data[0] == 0x1F && data[1] == 0x8B && data[2] == 0x08 && data[3] <= 0x1F && (data[8] == 0 || data[8] == 2 || data[8] == 4) && (data[9] <= 13 || data[9] == 0xFF) {
            return Some(0);
        }
    }

    None
}

//try to recognize exe/mzpe header
pub fn is_struct_mzpe(data: &[u8]) -> Option<usize> {

    if data.len() > 0x40 {

        //check for MZ magic
        if data[0] == 0x4D && data[1] == 0x5A { 

            //get dword from 0x3C offset and check if is there PE\x00\x00
            let pe_offset = u32_from_bytes_unchecked(&data[0x3c..], true) as usize;
            if pe_offset < data.len() && data[pe_offset] == 0x50 && data[pe_offset + 1] == 0x45 && data[pe_offset + 2] == 0 && data[pe_offset + 3] == 0 {
                return Some(0);
            }
        }
    }

    None
}

//try to recognize elf header
pub fn is_struct_elf(data: &[u8]) -> Option<usize> {

    if data.len() > 0x40 {

        //check for ELF magic
        if data[0] == 0x7F && data[1] == 0x45 && data[2] == 0x4C && data[3] == 0x46 {

            //addition check for 32/64 bit flag, endianness and version
            if (data[4] == 1 || data[4] == 2) && (data[5] == 1 || data[5] == 2) && data[6] == 1 { 
                return Some(0);
            }
        }
    }

    None
}
