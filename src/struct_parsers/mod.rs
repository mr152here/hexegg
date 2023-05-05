use std::mem::size_of;
pub mod struct_bmp;
pub mod struct_elf;
pub mod struct_gif;
pub mod struct_ico;
pub mod struct_mzpe;
pub mod struct_png;

pub struct FieldDescription {
    pub name: String,
    pub offset: usize,
    pub size: usize
}

pub fn parse_struct_by_name(data: &[u8], name: &str) -> Result<Vec<FieldDescription>, String> {
    match name {
        "bmp" => struct_bmp::parse_bmp_struct(data),
        "dib" => struct_bmp::parse_dib_struct(data),
        "elf" => struct_elf::parse_elf_struct(data),
        "ico" => struct_ico::parse_ico_struct(data),
        "gif" => struct_gif::parse_gif_struct(data),
        "mz" => struct_mzpe::parse_mz_struct(data),
        "mzpe" => struct_mzpe::parse_mzpe_struct(data),
        "pe" => struct_mzpe::parse_pe_struct(data),
        "png" => struct_png::parse_png_struct(data),
        _ => Err("Unsupported header!".to_string()),
    }
}

pub fn string_from_u8(data: &[u8], offset: usize) -> Option<String> {
    if let Some((i,_)) = data.into_iter().skip(offset).enumerate().find(|(_,b)| b.is_ascii_control()) {
        let s = String::from_utf8_lossy(&data[offset..offset+i]).to_string();
        return (!s.is_empty()).then_some(s);
    }
    None
}

pub fn read_u8(data: &[u8], offset: usize) -> Option<u8> {
    data.get(offset).copied()
}

pub fn read_le_u16(data: &[u8], offset: usize) -> Option<u16> {
    (data.len() >= (offset + size_of::<u16>())).then_some(u16::from_le_bytes(data[offset..offset + size_of::<u16>()].try_into().unwrap()))
}

pub fn read_le_u32(data: &[u8], offset: usize) -> Option<u32> {
    (data.len() >= (offset + size_of::<u32>())).then_some(u32::from_le_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()))
}

pub fn read_le_u64(data: &[u8], offset: usize) -> Option<u64> {
    (data.len() >= (offset + size_of::<u64>())).then_some(u64::from_le_bytes(data[offset..offset + size_of::<u64>()].try_into().unwrap()))
}

pub fn read_be_u16(data: &[u8], offset: usize) -> Option<u16> {
    (data.len() >= (offset + size_of::<u16>())).then_some(u16::from_be_bytes(data[offset..offset + size_of::<u16>()].try_into().unwrap()))
}

pub fn read_be_u32(data: &[u8], offset: usize) -> Option<u32> {
    (data.len() >= (offset + size_of::<u32>())).then_some(u32::from_be_bytes(data[offset..offset + size_of::<u32>()].try_into().unwrap()))
}

pub fn read_be_u64(data: &[u8], offset: usize) -> Option<u64> {
    (data.len() >= (offset + size_of::<u64>())).then_some(u64::from_be_bytes(data[offset..offset + size_of::<u64>()].try_into().unwrap()))
}
