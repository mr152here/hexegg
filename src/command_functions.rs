use std::fs::File;
use std::io::{Write as _, Read};
use std::fmt::Write as _;
use std::process::{Command, Stdio};
use regex_lite::Regex;
use crate::location_list::{Location, LocationList};
use crate::file_buffer::FileBuffer;
use crate::ColorScheme;
use crate::config::{Config, HighlightStyle, ScreenPagingSize};
use crate::signatures::*;
use crate::struct_parsers::parse_struct_by_name;

//set file offset to one/all filebuffers
pub fn set_position(file_buffers: &mut [FileBuffer], active_fb: usize, position: usize, lock_buffers: bool) {
    if lock_buffers {
        file_buffers.iter_mut().for_each(|fb| fb.set_position(position));
    } else {
        file_buffers[active_fb].set_position(position);
    }
}

//find and returns location of the next patch
pub fn find_patch(fb: &FileBuffer, start_offset: usize) -> Result<usize, String> {

    if fb.patches().is_empty() {
        Err("Patch list is empty.".to_owned()) 

    } else {
        match fb.patches().iter().find(|(o,_)| *o > start_offset) {
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
            .map(|(o,_)|{(format!("{:08X}", o), *o)})
            .collect::<LocationList>())
    }
}

//helper function that finds first location of byte pattern in file buffer
fn find_pattern(fb: &[u8], pattern: &[u8]) -> Option<usize> {
    fb.windows(pattern.len())
        .enumerate()
        .find_map(|(o,w)| w.starts_with(pattern).then_some(o))
}

//helper function that returns all locations of byte pattern in file buffer
fn find_all_patterns(fb: &[u8], pattern: &[u8]) -> Vec<usize> {
    fb.windows(pattern.len())
        .enumerate()
        .filter_map(|(o,s)| s.starts_with(pattern).then_some(o))
        .collect()
}

//return first offset of the byte pattern from the current file position
pub fn find(fb: &[u8], start_offset: usize, b: &[u8]) -> Result<usize, String> {

    match find_pattern(&fb[start_offset..], b) {
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
pub fn find_all(fb: &[u8], b: &[u8]) -> Result<LocationList, String> {
    let pattern_offsets = find_all_patterns(fb, b);

    if !pattern_offsets.is_empty() {
        let b_len = b.len();

        Ok(pattern_offsets.iter()
                  .map(|&o| Location {name:format!("{:08X}", o), offset:o, size: b_len})
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

//find if there is any string at the position
pub fn find_string_at_position(fb: &[u8], position: usize) -> Option<(usize, usize)> {

    if let Some(b) = fb.get(position) {
        if (0x20..=0x7E).contains(b) {

            //find start/end of the string
            let (mut s, mut e) = (position, position);
            while let Some(b) = fb.get(e + 1) {
                if !(0x20..=0x7E).contains(b) {
                    break;
                }
                e += 1;
            }

            while let Some(b) = fb.get(s.saturating_sub(1)) {
                if !(0x20..=0x7E).contains(b) {
                    break;
                }

                s = s.saturating_sub(1);

                if s == 0 {
                    break;
                }
            }
            return Some((s,e));
        }
    }
    None
}

//find if there is any "ascii - unicode" string at the position
pub fn find_unicode_string_at_position(fb: &FileBuffer, position: usize) -> Option<(usize, usize)> {

    if let Some(b) = fb.get(position) {
        if (0x20..=0x7E).contains(&b) || b == 0 {

            //find end of the string
            let mut last_was_ascii = b != 0;
            let (mut s, mut e) = (position, position);

            while let Some(b) = fb.get(e + 1) {

                if (last_was_ascii && b !=0) || (!last_was_ascii && !(0x20..=0x7E).contains(&b)) {
                    break;
                }
                last_was_ascii = !last_was_ascii;
                e += 1;
            }

            //find start of the string
            last_was_ascii = b != 0;
            while let Some(b) = fb.get(s.saturating_sub(1)) {

                if (last_was_ascii && b !=0) || (!last_was_ascii && !(0x20..=0x7E).contains(&b)) {
                    break;
                }

                last_was_ascii = !last_was_ascii;
                s = s.saturating_sub(1);

                if s == 0 {
                    break;
                }
            }

            //if string starts with 0 byte or ends with ascii byte remove them
            if let Some(b) = fb.get(s) {
                if b == 0 &&  s < e {
                    s += 1;
                }
            }

            if let Some(b) = fb.get(e) {
                if (0x20..=0x7E).contains(&b) && s < e {
                    e -= 1;
                }
            }

            return (s < e).then_some((s,e));
        }
    }
    None
}

//returns location of the first string from current position in the buffer. String must match regex and must be at least min_size long
pub fn find_string(fb: &[u8], start_offset: usize, min_size: usize, regex: &str) -> Result<usize, String> {
    let data = &fb[start_offset..];
    let mut start_index: usize = 0;
    let mut in_string = false;

    let re = match Regex::new(regex) {
        Err(s) => return Err(s.to_string()),
        Ok(re) => re,
    };

    for (index, b) in data.iter().enumerate() {

        if (0x20..=0x7E).contains(b) {
            if !in_string {
                in_string = true;
                start_index = index;
            }
        } else if in_string {

            //process only strings longer or equal to minimal size
            if (index - start_index) >= min_size && re.is_match(std::str::from_utf8(&data[start_index..index]).unwrap()) {
                return Ok(start_index + start_offset);
            }
            in_string = false;
        }
    }

    //if data ends with string
    if in_string && (data.len() - start_index) >= min_size && re.is_match(std::str::from_utf8(&data[start_index..]).unwrap()) {
        return Ok(start_index + start_offset);
    }

    Err("Not found!".to_owned())
}

//returns location of the first "ascii-unicode" (2 bytes per char) string from current position in the file buffer. String must match regex and must be at least min_size long
pub fn find_unicode_string(fb: &[u8], start_offset: usize, min_size: usize, regex: &str) -> Result<usize, String> {
    let data = &fb[start_offset..];
    let mut tmp_s = String::with_capacity(64);
    let mut start_index: usize = 0;
    let mut in_string = false;
    let mut index: usize = 0;
    let min_size = 2*min_size;

    let re = match Regex::new(regex) {
        Err(s) => return Err(s.to_string()),
        Ok(re) => re,
    };

    while let Some(b1) = data.get(index) {
        let b2 = match data.get(index + 1) {
            Some(&b2) => b2,
            None => break,
        };

        if (0x20..=0x7E).contains(b1) && b2 == 0 {
            if !in_string {
                in_string = true;
                start_index = index;
            }
            index += 1;

        } else if in_string {

            //process only strings longer or equal to minimal size
            if (index - start_index) >= min_size {

                fb[start_index..index].iter()
                    .filter(|&b| *b != 0)
                    .for_each(|&b| tmp_s.push(b as char));

                if regex.is_empty() || re.is_match(&tmp_s) {
                    return Ok(start_index + start_offset);
                }
                tmp_s.clear();
            }
            in_string = false;
        }
        index += 1;
    }

    //if data ends with string
    if in_string && (data.len() - start_index) >= min_size {

        fb[start_index..index].iter()
            .filter(|&b| *b != 0)
            .for_each(|&b| tmp_s.push(b as char));

        if regex.is_empty() || re.is_match(&tmp_s) {
            return Ok(start_index + start_offset);
        }
    }

    Err("Not found!".to_owned())
}

//returns location of all strings that matches specified regex and have at least min_size in length
pub fn find_all_strings(fb: &[u8], min_size: usize, regex: &str) -> Result<LocationList, String> {
    let mut loc_list = LocationList::new();
    let mut start_index: usize = 0;
    let mut in_string = false;

    let re = match Regex::new(regex) {
        Err(s) => return Err(s.to_string()),
        Ok(re) => re,
    };

    for (index, b) in fb.iter().enumerate() {

        if (0x20..=0x7E).contains(b) {
            if !in_string {
                in_string = true;
                start_index = index;
            }
        } else if in_string {

            //process only strings with more then minimal size
            if (index - start_index) >= min_size && re.is_match(std::str::from_utf8(&fb[start_index..index]).unwrap()) {
                let s = String::from_utf8_lossy(&fb[start_index..index]).to_string();
                let s_len = s.len();
                loc_list.add_location(Location{name: s, offset: start_index, size: s_len});
            }
            in_string = false;
        }
    }

    //if data ends with string
    if in_string && (fb.len() - start_index) >= min_size && re.is_match(std::str::from_utf8(&fb[start_index..]).unwrap()) {
        let s = String::from_utf8_lossy(&fb[start_index..]).to_string();
        let s_len = s.len();
        loc_list.add_location(Location{name: s, offset: start_index, size: s_len});
    }
    
    match loc_list.is_empty() {
        true => Err("Not found!".to_owned()),
        false => Ok(loc_list),
    }
}

//returns location of all ascii-unicode strings that matches specified regex and have at least min_size in length
pub fn find_all_unicode_strings(fb: &[u8], min_size: usize, regex: &str) -> Result<LocationList, String> {
    let mut loc_list = LocationList::new();
    let mut tmp_s = String::with_capacity(64);
    let mut start_index: usize = 0;
    let mut in_string = false;
    let mut index: usize = 0;
    let min_size = 2 * min_size;

    let re = match Regex::new(regex) {
        Err(s) => return Err(s.to_string()),
        Ok(re) => re,
    };

    while let Some(b1) = fb.get(index) {
        let b2 = match fb.get(index + 1) {
            Some(&b2) => b2,
            None => break,
        };

        if (0x20..=0x7E).contains(b1) && b2 == 0 {
            if !in_string {
                in_string = true;
                start_index = index;
            }
            index += 1;

        } else if in_string {

            //process only strings longer or equal to minimal size
            if (index - start_index) >= min_size {

                fb[start_index..index].iter()
                    .filter(|&b| *b != 0)
                    .for_each(|&b| tmp_s.push(b as char));

                if regex.is_empty() || re.is_match(&tmp_s) {
                    loc_list.add_location(Location{name: tmp_s.clone(), offset: start_index, size: index - start_index});
                }
                tmp_s.clear();
            }
            in_string = false;
        }
        index += 1;
    }

    //if data ends with string
    if in_string && (fb.len() - start_index) >= min_size {

        fb[start_index..index].iter()
            .filter(|&b| *b != 0)
            .for_each(|&b| tmp_s.push(b as char));

        if regex.is_empty() || re.is_match(&tmp_s) {
            loc_list.add_location(Location{name: tmp_s.clone(), offset: start_index, size: index - start_index});
        }
    }

    match loc_list.is_empty() {
        true => Err("Not found!".to_owned()),
        false => Ok(loc_list),
    }
}

//find and returns first diff from current file position
pub fn find_diff(file_buffers: &[FileBuffer], start_offset: usize, active_fb_index: usize) -> Option<usize> {

    file_buffers[active_fb_index].iter()
        .enumerate()
        .skip(start_offset)
        .find_map(|(o, byte)| {
            file_buffers.iter()
                .any(|filebuf| { if let Some(b) = filebuf.get(o) { b != *byte } else { true } })
                .then_some(o)
        })
}

//find and returns list of all diffs
pub fn find_all_diffs(file_buffers: &[FileBuffer], active_fb_index: usize) -> Result<LocationList, String> {

    let ll = file_buffers[active_fb_index].iter()
        .enumerate()
        .filter(|&(offset, byte)| {
            file_buffers.iter().any(|filebuf| { if let Some(b) = filebuf.get(offset) { b != *byte } else { true } })
        })
        .map(|(o,_)| (format!("{:08X}", o), o))
        .collect::<LocationList>();

    match ll.is_empty() {
        true => Err("Not found!".to_owned()),
        false => Ok(ll),
    }
}

//find all headers and structs
pub fn find_all_signatures(file_buffers: &[FileBuffer], active_fb_index: usize, signature_names: Option<Vec<String>>, ignored: bool) -> Result<LocationList, String> {

    let mut result_ll = LocationList::new();
    let file_len = file_buffers[active_fb_index].len();
    let file_slice = &file_buffers[active_fb_index];

    for i in 0..file_len {
        let tmp_file_slice = &file_slice[i..];

        if let Some(sig_name) = get_signature(tmp_file_slice) {
            if let Some(ref name_list) = signature_names {
                let matched = name_list.iter().any(|name| name.eq(sig_name));
                
                if !matched && ignored || matched && !ignored {
                    result_ll.add_location(Location{name: sig_name.to_owned(), offset: i, size: 0});
                }
            } else {
                result_ll.add_location(Location{name: sig_name.to_owned(), offset: i, size: 0});
            }
        }
    }

    match result_ll.is_empty() {
        true => Err("Not found!".to_owned()),
        false => Ok(result_ll),
    }
}

//find all boomarks
pub fn find_all_bookmarks(file_buffers: &[FileBuffer], active_fb_index: usize) -> Result<LocationList, String> {
    let fb = &file_buffers[active_fb_index];
    let ll = (0..10).filter_map(|idx| fb.bookmark(idx)
        .map(|o| (format!("bm_{}",idx), o)))
        .collect::<LocationList>();

    match ll.is_empty() {
        true => Err("No bookmarks set.".to_owned()),
        false => Ok(ll),
    }
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

//replace all bytes from location list with bytes from pattern or selected block
pub fn replace_all(fb: &mut FileBuffer, bytes: Vec<u8>) -> Result<(), String> {

    //get pattern or selected block
    let bytes = if bytes.is_empty() {
        if let Some((s,e)) = fb.selection() {
            fb[s..=e].to_vec()
        } else {
            return Err("No pattern or block specified!".to_owned());
        }
    } else {
        bytes
    };

    //check if all results in location list are the same size as pattern/block
    let ll = (*fb.location_list()).clone();
    if ll.is_empty() {
        return Err("No locations in location bar!".to_owned());

    } else if (&ll).into_iter().any(|loc| loc.size != bytes.len()) {
        return Err("All results must be the same size as specified pattern or block!".to_owned());
    }

    //iterate over locations and create patches
    for loc in ll {
        let mut offset = loc.offset;
        //TODO: creating patched this way is relative slow and expensive because of set()
        bytes.iter().for_each(|b| {fb.set(offset, *b); offset += 1;});
    }

    Ok(())
}

//split file into blocks with size of block_size and calculate entropy of each.
//if entropy of next block is in abs less than margin, blocks are merged. Returns list of all blocks that remains
pub fn calculate_entropy(fb: &[u8], block_size: usize, margin: f32) -> LocationList {
    let mut prev_ent = 9999.0;

    fb.chunks(block_size)
        .filter(|c| c.len() == block_size)
        .enumerate()
        .filter_map(|(i,c)|{
            let ent = (100.0 * entropy(c)).round() / 100.0;
            if (prev_ent - ent).abs() > margin {
                prev_ent = ent;
                return Some((format!(" {:.2}", ent), i*block_size));
            }
            None
        })
        .collect::<LocationList>()
}

//calculate histogram and return sorted LocationList
pub fn calculate_histogram(data: &[u8]) -> LocationList {
    let mut histogram = [(0,0); 256];
    data.iter().for_each(|&b| histogram[b as usize].1 += 1);
    histogram.iter_mut().enumerate().for_each(|(i,(b,_))| *b = i);
    histogram.sort_by_key(|&(_,v)| usize::MAX - v);

    histogram.iter()
        .map(|(b,v)| (format!("{:02X}_{}", b, v), 0))
        .collect::<LocationList>()
}

//parse binary structure at the given offset.
pub fn parse_struct(data: &[u8], name: Option<String>) -> Result<LocationList, String> {

    let sig_name = match name {
        Some(s) => s,
        None => {
            match get_signature(data) {
                Some(s) => s.to_string(),
                None => return Err("Unknown header signature!".to_string()),
            }
        },
    };

    parse_struct_by_name(data, &sig_name)
}

//save selected block into the file
pub fn save_block(file_buffers: &[FileBuffer], active_fb_index: usize, file_name: &str) -> Result<String, String> {

    if let Some((start,end)) = file_buffers[active_fb_index].selection() {
        return match save_file(file_name, &file_buffers[active_fb_index][start..=end], true) {
            Ok(count) => Ok(format!("written {} bytes to '{}'.", count, file_name)),
            Err(s) => Err(s),
        };
    }
    Err("Please select the block first.".to_owned())
}

//create a new filebuffer from selected/yanked block
pub fn open_block(file_buffers: &mut Vec<FileBuffer>, active_fb_index: usize, yank_buffer: &[u8]) -> Result<(), String> {

    if let Some((s,e)) = file_buffers[active_fb_index].selection() {
        let file_data = file_buffers[active_fb_index][s..=e].to_vec();
        let mut fb = FileBuffer::from_vec(file_data);

        fb.set_filename(format!("dump_{:08X}_{:08X}", s, e).as_str());
        file_buffers.push(fb);

    } else if !yank_buffer.is_empty() {
        let mut fb = FileBuffer::from_vec(yank_buffer.to_vec());
        fb.set_filename("dump_yanked");
        file_buffers.push(fb);

    } else {
        return Err("Please select or yank the block first.".to_owned());
    }
    Ok(())
}

//export selected block into the txt file
pub fn export_block(file_buffers: &[FileBuffer], active_fb_index: usize) -> Result<String, String> {

    if let Some((start,end)) = file_buffers[active_fb_index].selection() {
        let block_chunks = file_buffers[active_fb_index][start..=end].chunks(16);
        let mut out_string = String::with_capacity(5*(end - start + 1));

        for chunk in block_chunks {
            for byte in chunk {
                if let Err(s) = write!(&mut out_string, "0x{:02X}, ", byte) {
                    return Err(s.to_string());
                }
            }
            out_string.push('\n');
        }

        let file_name = format!("export_{:08X}.txt", start);
        return match save_file(&file_name, out_string.as_bytes(), true) {
            Ok(count) => Ok(format!("written {} bytes to '{}'.", count, file_name)),
            Err(s) => Err(s),
        }
    }
    Err("Please select the block first.".to_owned())
}

//try to send a data via stdin to external application
pub fn pipe_block_to_program(data: &[u8], program_name: &Vec<String>) -> Result<(), String> {

    if let Some(prog_name) = program_name.first() {
        if !prog_name.is_empty() {

            let prog_args = if program_name.len() > 1 {
                &program_name.as_slice()[1..]
            } else {
                &[]
            };

            //spawn child process
            match Command::new(prog_name).stdin(Stdio::piped()).args(prog_args).spawn() {
                Ok(mut child) => {
                    if let Some(mut si) = child.stdin.take() {
                        if let Err(e) = si.write(data) {
                            return Err(format!("Can't write to the stdin of the {}. {}", prog_name, e));
                        }
                    }

                    //wait for child process to finish
                    return match child.wait() {
                        Err(e) => Err(format!("{}", e)),
                        Ok(_) => Ok(()),
                    };
                },
                Err(e) => return Err(format!("Can't spawn '{}'! {}", prog_name, e)),
            }
        }
    }
    Ok(())
}

//try to read data from stdin.
pub fn read_stdin(size_limit: Option<u64>) -> Result<Vec<u8>, String> {
    let mut v = Vec::<u8>::new();
    match size_limit {
        Some(limit) => {
            if let Err(s) = std::io::stdin().lock().take(limit).read_to_end(&mut v) {
                return Err(format!("Unable to read data from STDIN. {}", s));
            }
        },
        None => {
            if let Err(s) = std::io::stdin().lock().read_to_end(&mut v) {
                return Err(format!("Unable to read data from STDIN. {}", s));
            }
        },
    }
    Ok(v)
}

//open and read file into Vec<u8>.
pub fn read_file(file_name: &str, size_limit: Option<u64>) -> Result<Vec<u8>, String> {

    let f = match std::fs::File::open(file_name) {
        Err(s) => return Err(s.to_string()),
        Ok(f) => f,
    };

    let size_limit = match size_limit {
        Some(l) => l,
        None => {
            match f.metadata() {
                Err(s) => return Err(format!("Unable to obtain metadata from file '{}'. {}", file_name, s)),
                Ok(md) => md.len(),
            }
        }
    };

    let mut buf = Vec::<u8>::new();
    match f.take(size_limit).read_to_end(&mut buf) {
        Err(s) => Err(s.to_string()),
        Ok(_) => Ok(buf),
    }
}

//save filebuffer/data into the file with name filename
pub fn save_file(file_name: &str, data: &[u8], truncate: bool) -> Result<usize, String> {

    let mut file = match File::options().write(true).create(true).truncate(truncate).open(file_name) {
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
