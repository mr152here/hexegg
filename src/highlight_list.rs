use crossterm::style::Color;

pub struct HighlightList {
    highlights: Vec<(usize, Option<Color>)>,
}


impl HighlightList {

    pub fn new() -> HighlightList {
        HighlightList { highlights: vec![(0,None)] }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (usize, Option<Color>)> {
        self.highlights.iter()
    }

    fn highlight_idx(&self, offset: usize) -> Option<usize> {
        let mut low_idx = 0;
        let mut high_idx = self.highlights.len();
        let mut mid_idx = high_idx / 2;

        while let Some(&(s,_)) = self.highlights.get(mid_idx) {

            if offset >= s {

                //get offset from the next element
                if let Some(&(next_s,_)) = self.highlights.get(mid_idx+1) {

                    //if it is between them, then mid_idx is target idx
                    if offset < next_s {
                        return Some(mid_idx);
                    //if not, go to the right side of the tree
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
    pub fn add(&mut self, start_offset: usize, end_offset: usize, color: Option<Color>) {

        //find index where new range should be
        if let Some(idx1) = self.highlight_idx(start_offset) {
            let mut to_remove = 0;

            if let Some(idx2) = self.highlight_idx(end_offset + 1) {
                let c = self.highlights[idx2].1;

                if color.is_some() || c.is_some() {
                    self.highlights.insert(idx2 + 1, (end_offset + 1, c));
                }
                to_remove = idx2 - idx1;
            }

            //if it is the same offset as original one, update the color
            let didx = if self.highlights[idx1].0 == start_offset {
                self.highlights[idx1].1 = color;
                1

            //if the new interval's color is None and IDX1 is None. Just do nothing.
            } else if color.is_none() && self.highlights[idx1].1.is_none() {
                1

            //otherwise insert a new element
            } else {
                self.highlights.insert(idx1 + 1, (start_offset, color));
                0
            };

            //remove all ranges that are overlaped by the new one
            for _ in 0..to_remove {
                self.highlights.remove(idx1 + 2 - didx);
            }
        }
    }

    pub fn color(&self, offset: usize) -> Option<Color> {
        match self.highlight_idx(offset) {
            Some(idx) => self.highlights[idx].1,
            None => None,
        }
    }

    //returns highlighted interval for offset. If offset is not highlighted (color is None) return None,
    //if the range is the last one. The end offset is set to usize::MAX
    pub fn range(&self, offset: usize) -> Option<(usize, usize)> {
        if let Some(idx) = self.highlight_idx(offset) {
            let &(start_offset, c) = self.highlights.get(idx).unwrap();

            if c.is_some() {
                let &(end_offset,_) = self.highlights.get(idx + 1).unwrap_or(&(usize::MAX, None));
                return Some((start_offset, end_offset - 1));
            }
        }
        None
    }

    //clear all highlights
    pub fn clear(&mut self) {
        self.highlights.clear();
        self.highlights.push((0, None));
    }
}

impl FromIterator<(usize, usize, Option<Color>)> for HighlightList {

    fn from_iter<I: IntoIterator<Item=(usize, usize, Option<Color>)>>(iter: I) -> Self {
        let mut hl = HighlightList::new();
        iter.into_iter().for_each(|(s,e,c)| hl.add(s,e,c));
        hl
    }
}
