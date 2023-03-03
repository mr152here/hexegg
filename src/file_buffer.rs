use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::cmp::min;
use crossterm::style::Color;
use crate::location_list::LocationList;

pub struct FileBuffer {
    file_name: String,
    file_data: Vec<u8>,
    patch_map: HashMap<usize, u8>,
    current_position: usize,
    selection: Option<(usize, usize)>,
    //highlights: Vec<(usize, usize, Color)>,
    highlights2: Vec<(usize, Option<Color>)>,
    bookmarks: [Option<usize>; 10],
    location_list: LocationList,
    original_hash: u64,
    modified: bool
}


impl FileBuffer {

    pub fn from_vec(v: Vec<u8>) -> FileBuffer {
        let mut hasher = DefaultHasher::new();
        hasher.write(&v);

        FileBuffer { 
            file_name: "undefined_filename".to_owned(),
            file_data: v,
            patch_map: HashMap::new(),
            current_position: 0,
            selection: None,
            //highlights: Vec::new(),
            highlights2: vec![(0,None)],
            bookmarks: [None; 10],
            location_list: LocationList::new(),
            original_hash: hasher.finish(),
            modified: false
        }
    }

    pub fn filename(&self) -> &str {
        &self.file_name
    }

    pub fn set_filename(&mut self, file_name: &str) {
        self.file_name = file_name.to_owned();
    }

    pub fn position(&self) -> usize {
        self.current_position
    }

    pub fn set_position(&mut self, position: usize) {
        self.current_position = position; 
    }

    pub fn len(&self) -> usize {
        self.file_data.len()
    }

    //returns byte on given offset (if any)
    pub fn get(&self, offset: usize) -> Option<u8> {
        self.file_data.get(offset).cloned()
    }

    //return internal data vector as a reference to slice
    pub fn as_slice(&self) -> &[u8] {
        self.file_data.as_slice()
    }

    //change a byte in the file buffer and add original one to the patch map
    pub fn set(&mut self, offset: usize, value: u8) {
        if let Some(byte) = self.file_data.get_mut(offset) {
            if *byte != value {
                self.patch_map.entry(offset).or_insert(*byte);
                *byte = value;

                let mut hasher = DefaultHasher::new();
                hasher.write(&self.file_data);
                self.modified = self.original_hash != hasher.finish();
            }

        //append byte if is one past the end of the file. Original byte in patch is set to 0
        } else if offset == self.file_data.len() {
            self.file_data.push(value);
            self.patch_map.entry(offset).or_insert(0);

            let mut hasher = DefaultHasher::new();
            hasher.write(&self.file_data);
            self.modified = self.original_hash != hasher.finish();
        }
    }

    //delete bytes from file data (defined by selection) and recalculate affected patches
    pub fn remove_block(&mut self) {
        if let Some((s,e)) = self.selection {
            if s < self.file_data.len() {
                let e = min(e, self.file_data.len() - 1);
                self.file_data.splice(s..=e, []);

                //recalculate offset of all existing patches, and remove patches from deleted block
                if self.patch_map.iter().any(|(o,_)| *o >= s) {
                    let selection_len = e - s + 1;
                    self.patch_map = self.patch_map.iter()
                        .filter(|(o,_)| *o < &s || *o > &e )
                        .map(|(o,b)| (*o - if *o >= e {selection_len} else {0}, *b))
                        .collect();
                }

                let mut hasher = DefaultHasher::new();
                hasher.write(&self.file_data);
                self.modified = self.original_hash != hasher.finish();
            }
        }
    }

    //insert byte vector into the file buffer at given position. Recalculate existing patches
    pub fn insert_block(&mut self, position: usize, data: Vec<u8>) {
        if !data.is_empty() && position <= self.file_data.len() {

            let data_len = data.len();
            self.file_data.splice(position..position, data);

            //recalculate offset of existing patches
            if self.patch_map.iter().any(|(o,_)| *o >= position) {
                self.patch_map = self.patch_map.iter()
                    .map(|(o,b)| (*o + if *o >= position { data_len } else { 0 }, *b))
                    .collect();
            }

            //add all new bytes as patches with "original value" 0
            let i = self.file_data.iter().skip(position);
            for (idx,_) in i.enumerate() {
                if idx == data_len {
                    break;
                }
                self.patch_map.entry(position + idx).or_insert(0);
            }

            let mut hasher = DefaultHasher::new();
            hasher.write(&self.file_data);
            self.modified = self.original_hash != hasher.finish();
        }
    }

    //recalculate and set a new data hash. 
    pub fn reset_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        hasher.write(&self.file_data);
        self.original_hash = hasher.finish();
        self.modified = false;
    }

    //returns true if byte at offset is modified
    pub fn is_patched(&self, offset: usize) -> bool {
        self.patch_map.get(&offset).is_some()
    }

    //returns true if something in the file buffer was changed
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    //returns stored selection interval
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    //set selection interval.
    pub fn set_selection(&mut self, selection: Option<(usize, usize)>) {
        self.selection = match selection {
            None => None,
            Some((s,e)) => {
                let data_len = self.file_data.len().saturating_sub(1);
                let start = std::cmp::min(s, data_len);
                let end = std::cmp::min(e, data_len);
                Some((std::cmp::min(start,end), std::cmp::max(start,end)))
            },
        };
    }

    //returns true if offset is inside the selection range
    pub fn is_selected(&self, offset: usize) -> bool {
        match self.selection {
            Some((s,e)) => (s..=e).contains(&offset),
            None => false,
        }
    }

    //return IDX where offset belong to (the left)
    fn highlight_idx(&self, offset: usize) -> Option<usize> {
        let mut low_idx = 0;
        let mut high_idx = self.highlights2.len();
        let mut mid_idx = high_idx / 2;

        while let Some(&(s,_)) = self.highlights2.get(mid_idx) {
                            
            if offset >= s {

                //get offset from the next element
                if let Some(&(next_s,_)) = self.highlights2.get(mid_idx+1) {

                    //if it is between them mid_idx is target idx
                    if offset < next_s {
                        return Some(mid_idx);
                    //if not, we must go the right side of the tree
                    } else {
                        low_idx = mid_idx; 
                    }

                //if we are the last element in the vector return mid_idx
                } else {
                    return Some(mid_idx);
                }

            //check the left side of the tree
            } else {
                high_idx = mid_idx;
            }
            mid_idx = (high_idx + low_idx) / 2;
        }
        None
    }

    //add highlighted interval to the list
    pub fn add_highlight(&mut self, start_offset: usize, end_offset: usize, color: Color) {

        //find index where new range should be
        if let Some(idx1) = self.highlight_idx(start_offset) {
            let mut to_remove = 0;

            if let Some(idx2) = self.highlight_idx(end_offset + 1) {
                self.highlights2.insert(idx2 + 1, (end_offset + 1, self.highlights2[idx2].1));
                to_remove = idx2-idx1;
            }

            self.highlights2.insert(idx1 + 1, (start_offset, Some(color)));

            //remove all ranges that overlaped be new one
            for _ in 0..to_remove {
                self.highlights2.remove(idx1 + 2);
            }
        }
    }

    //clear highlighting for interval
    pub fn remove_highlight(&mut self, _start_offset: usize, _end_offset: usize) {
        //TODO:
    }


    //clear all highlighted intervals
    pub fn clear_highlights(&mut self) {
        self.highlights2.clear();
        self.highlights2.push((0, None));
    }

    //find if offset is highlighted and return it color.
    pub fn get_highlight(&self, offset: usize) -> Option<Color> {
        match self.highlight_idx(offset) {
            Some(idx) => self.highlights2[idx].1,
            None => None,
        }
    }

    pub fn set_bookmark(&mut self, idx: usize, offset: Option<usize>) {
        if let Some(bookmark) = self.bookmarks.get_mut(idx) {
            *bookmark = offset;
        }
    }

    pub fn bookmark(&self, idx: usize) -> Option<usize> {
        match self.bookmarks.get(idx) {
            Some(&u) => u,
            None => None,
        }
    }

    //restore original byte in file buffer from patch map
    pub fn unpatch_offset(&mut self, offset: usize) {
        if let Some(orig_byte) = self.patch_map.remove(&offset) {
            if let Some(byte) = self.file_data.get_mut(offset) {
                *byte = orig_byte;

                let mut hasher = DefaultHasher::new();
                hasher.write(&self.file_data);
                self.modified = self.original_hash != hasher.finish();
            }
        }
    }

    //delete all patches
    pub fn clear_patches(&mut self) {
        self.patch_map.clear();
    }
    
    //returns sorted list of all patches
    pub fn patches(&self) -> Vec<(usize, u8)> {
        let mut v = self.patch_map.iter()
                    .map(|(&k, &v)| (k, v))
                    .collect::<Vec<(usize,u8)>>();

        v.sort_by_key(|t| t.0);
        v
    }

    //returns actual location list
    pub fn location_list(&self) -> &LocationList {
        &self.location_list
    }

    //returns mutable reference to location list
    pub fn location_list_mut(&mut self) -> &mut LocationList {
        &mut self.location_list
    }

    //set location list
    pub fn set_location_list(&mut self, location_list: LocationList) {
        self.location_list = location_list;
    }
}

