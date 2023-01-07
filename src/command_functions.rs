use std::{fs,str};
use std::fs::File;
use std::io::Write;
use crate::location_list::LocationList;
use crate::file_buffer::FileBuffer;
use crate::ColorScheme;
use crate::config::{Config, HighlightStyle, ScreenPagingSize};
use crate::struct_finder::*;

//set file offset to one/all filebuffers
pub fn set_position(file_buffers: &mut [FileBuffer], active_fb: usize, position: usize, lock_buffers: bool) {
    if lock_buffers {
        file_buffers.iter_mut().for_each(|fb| fb.set_position(position));
    } else {
        file_buffers[active_fb].set_position(position);
    }
}

//find and returns next patch
pub fn find_patch(fb: &FileBuffer) -> Result<usize, String> {

    if fb.patches().is_empty() {
        Err("Patch list is empty.".to_owned()) 

    } else {
        let position = fb.position();
        match fb.patches().iter().find(|(o,_)| *o > position) {
            Some((pos,_)) => Ok(*pos),
            None => Err("Can't find next patch.".to_owned()),
        }
    }
}

//returns vector of all patches in filebuffer
pub fn find_all_patches(fb: &FileBuffer) -> Result<LocationList, String> {

    if fb.patches().is_empty() {
        Err("Patch list is empty.".to_owned()) 

    } else {
        Ok(fb.patches().iter()
            .map(|(o,_)|{(*o, format!("{:08X}", o))})
            .collect::<LocationList>())
    }
}

//helper function that finds first location of byte pattern in file buffer
fn find_pattern(buffer: &[u8], pattern: &[u8]) -> Option<usize> {
    buffer.windows(pattern.len())
        .enumerate()
        .find_map(|(o,w)| w.starts_with(pattern).then_some(o))
}

//helper function that returns all locations of byte pattern in file buffer
fn find_all_patterns(buffer: &[u8], pattern: &[u8]) -> Vec<usize> {
    buffer.windows(pattern.len())
        .enumerate()
        .filter_map(|(o,s)| s.starts_with(pattern).then_some(o))
        .collect()
}

//return first offset of the byte pattern from the current file position
pub fn find(fb: &FileBuffer, b: &[u8]) -> Result<usize, String> {
    let start_offset = fb.position() + 1;

    match find_pattern(&fb.as_slice()[start_offset..], b) {
        Some(o) => Ok(o + start_offset),
        None => {
            let mut s = String::from("Pattern ");
            b.iter().for_each(|&byte| {
                if byte.is_ascii_graphic() {
                    s.push_str(format!("{}", char::from_u32(byte as u32).unwrap()).as_str())
                } else {
                    s.push_str(format!("\\x{:02X}", byte).as_str())
                }
            });
            s.push_str(" not found!");
            Err(s)
        },
    }
}

//find all occurances of b in filebuffer
pub fn find_all(fb: &FileBuffer, b: &[u8]) -> Result<LocationList, String> {
    let pattern_offsets = find_all_patterns(fb.as_slice(), b);

    if !pattern_offsets.is_empty() {
        Ok(pattern_offsets.iter()
                  .map(|&o| (o, format!("{:08X}", o)))
                  .collect::<LocationList>())
    } else {
        let mut s = String::from("Pattern ");
        b.iter().for_each(|&byte| {
            if byte.is_ascii_graphic() {
                s.push_str(format!("{}", char::from_u32(byte as u32).unwrap()).as_str())
            } else {
                s.push_str(format!("\\x{:02X}", byte).as_str())
            }
        });
        s.push_str(" not found!");
        Err(s)
    }
}

//returns location of first string from current position in filebuffer. String must contains
//substring and must be at least min_size long
pub fn find_string(fb: &FileBuffer, min_size: usize, substring: &Vec<u8>) -> Result<usize, String> {
    let data_start_offset = fb.position() + 1;
    let data = &fb.as_slice()[data_start_offset..];
    let mut start_index: usize = 0;
    let mut in_string = false;

    for (index, b) in data.iter().enumerate() {

        if *b >= 0x20 && *b <= 0x7E { 
            if !in_string {
                in_string = true;
                start_index = index;
            }
        } else if in_string {

            //process only strings longer or equal to minimal size
            if (index - start_index) >= min_size {
                if substring.is_empty() || data[start_index..index].windows( substring.len() ).any(|s| s.starts_with(substring)) {
                    return Ok(start_index + data_start_offset);
                }
            }
            in_string = false;
        }
    }

    //if data ends with string
    if in_string && (data.len() - start_index) >= min_size { 
        if substring.is_empty() || data[start_index..].windows( substring.len() ).any(|s| s.starts_with(substring)) {
            return Ok(start_index + data_start_offset);
        }
    }

    Err("Not found!".to_owned())
}

//returns location of all strings that contains specified substring and have at least min_size in length
pub fn find_all_strings(fb: &FileBuffer, min_size: usize, substring: &Vec<u8>) -> LocationList {
    let data = fb.as_slice();
    let mut loc_list = LocationList::new();
    let mut start_index: usize = 0;
    let mut in_string = false;

    for (index, b) in data.iter().enumerate() {

        if *b >= 0x20 && *b <= 0x7E { 
            if !in_string {
                in_string = true;
                start_index = index;
            }
        } else if in_string {

            //process only strings with more then minimal size
            if (index - start_index) >= min_size {
                if substring.is_empty() || data[start_index..index].windows( substring.len() ).any(|s| s.starts_with(substring)) {
                    loc_list.add_location(start_index, String::from_utf8_lossy(&data[start_index..index]).to_string());
                }
            }
            in_string = false;
        }
    }

    //if data ends with string
    if in_string && (data.len() - start_index) >= min_size { 
        if substring.is_empty() || data[start_index..].windows( substring.len() ).any(|s| s.starts_with(substring)) {
            loc_list.add_location(start_index, String::from_utf8_lossy(&data[start_index..]).to_string());
        }
    }
    
    loc_list
}

//find and returns first diff from current file position
pub fn find_diff(file_buffers: &[FileBuffer], active_fb_index: usize) -> Option<usize> {
    let position = &file_buffers[active_fb_index].position() + 1;

    file_buffers[active_fb_index].as_slice()
        .iter()
        .enumerate()
        .skip(position)
        .find_map(|(o, byte)| {
            file_buffers.iter()
                .any(|filebuf| { if let Some(b) = filebuf.get(o) { b != *byte } else { true } })
                .then_some(o)
        })
}

//find and returns list of all diffs
pub fn find_all_diffs(file_buffers: &[FileBuffer], active_fb_index: usize) -> LocationList {

    file_buffers[active_fb_index].as_slice()
        .iter()
        .enumerate()
        .filter(|&(offset, byte)| {
            file_buffers.iter().any(|filebuf| { if let Some(b) = filebuf.get(offset) { b != *byte } else { true } })
        })
        .map(|(o,_)| (o, format!("{:08X}", o)))
        .collect::<LocationList>()
}

//find all headers and structs
//TODO: implement block size to location list to fill it here!!
pub fn find_all_headers(file_buffers: &[FileBuffer], active_fb_index: usize) -> LocationList {

    let mut result_ll = LocationList::new();
    let file_len = file_buffers[active_fb_index].len();
    let file_slice = file_buffers[active_fb_index].as_slice();

    for i in 0..file_len {
        let tmp_file_slice = &file_slice[i..];

        //TODO: block size to location list
        if let Some(_) = is_struct_bmp(tmp_file_slice) {
            result_ll.add_location(i, "bmp".to_owned());

        } else if let Some(_) = is_struct_png(tmp_file_slice) {
            result_ll.add_location(i, "png".to_owned());
        
        } else if let Some(_) = is_struct_ico(tmp_file_slice) {
            result_ll.add_location(i, "ico".to_owned());

        } else if let Some(_) = is_struct_gif(tmp_file_slice) {
            result_ll.add_location(i, "gif".to_owned());
        }
    }

    result_ll
}

//helper function that calculate entropy of data block
fn entropy(data: &[u8]) -> f32 {
    let data_len = data.len() as f32;
    let mut ent: f32 = 0.0;
    let mut histogram = [0; 256];

    data.iter().for_each(|&b| histogram[b as usize] += 1);

    histogram.into_iter()
        .filter(|&c| c > 0)
        .for_each(|count| {
            let c = count as f32 / data_len;
            ent = c.mul_add(c.log2(), ent);
        });

    -ent
}

//split file into blocks with size of block_size and calculate entropy of each.
//if entropy of next block is in abs less than margin, blocks are merged. Returns list of all blocks that remains
pub fn calculate_entropy(fb: &FileBuffer, block_size: usize, margin: f32) -> LocationList {
    let mut prev_ent = 9999.0;

    fb.as_slice()
        .chunks(block_size)
        .filter(|c| c.len() == block_size)
        .enumerate()
        .filter_map(|(i,c)|{
            let ent = (100.0 * entropy(c)).round() / 100.0;
            if (prev_ent - ent).abs() > margin {
                prev_ent = ent;
                return Some((i*block_size, format!("{:>8.2}", ent)));
            }
            None
        })
        .collect::<LocationList>()
}

//open and read file into Vec<u8>.
pub fn read_file(file_name: &String) -> Result<Vec<u8>, String> {
    match fs::read(file_name) {
        Ok(data) => Ok(data),
        Err(s) => Err(s.to_string()),
    }
}

//save filebuffer/data into the file with name filename
pub fn save_file(file_name: &str, data: &[u8]) -> Result<usize, String> {
    let mut file = match File::create(file_name) {
        Ok(f) => f,
        Err(s) => return Err(s.to_string()),
    };

    let count = match file.write(data) {
        Ok(c) => c,
        Err(s) => return Err(s.to_string()),
    };

    Ok(count)
}

//set internal variable during program execution. Are the same as from config file.
pub fn set_variable(var_name: &str, var_value: &str, config: &mut Config, color_scheme: &mut ColorScheme) -> Result<(), String> {
    match var_name {
        "colorscheme" => {
            match config.color_scheme(var_value) {
                Some(cs) => *color_scheme = cs.clone(),
                None => return Err(format!("Unknown color scheme '{}'",var_value))
            }
        },

        "highlightstyle" => {
            match HighlightStyle::from_str(var_value) {
                Some(hs) => config.highlight_style = hs,
                None => return Err(format!("Unknown highlight style '{}'",var_value))
            }
        },

        "screenpagingsize" => {
            match ScreenPagingSize::from_str(var_value) {
                Some(hs) => config.screen_paging_size = hs,
                None => return Err(format!("Unknown screen paging size '{}'",var_value))
            }
        },
        _ => return Err(format!("Unknown variable '{}'", var_name)),
    }
    Ok(())
}

