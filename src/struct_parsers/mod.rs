pub mod struct_bmp;
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
        "ico" => struct_ico::parse_ico_struct(data),
        "gif" => struct_gif::parse_gif_struct(data),
        "mz" => struct_mzpe::parse_mz_struct(data),
        "mzpe" => struct_mzpe::parse_mzpe_struct(data),
        "pe" => struct_mzpe::parse_pe_struct(data),
        "png" => struct_png::parse_png_struct(data),
        _ => Err("Unsupported header!".to_string()),
    }
}
