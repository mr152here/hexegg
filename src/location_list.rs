pub struct LocationList {
    current_index: usize,
    loc_list: Vec<(usize, String)>
}

impl LocationList {

    pub fn new() -> LocationList {
        LocationList {
            current_index: 0,
            loc_list: Vec::<(usize, String)>::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.loc_list.is_empty()
    }

    //returns next location
    pub fn next(&mut self) -> Option<(usize, String)> {
        if let Some(v) = self.get(self.current_index + 1) {
            self.current_index += 1;
            return Some(v);
        } 
        self.get(self.current_index)
    }

    //returns previous location
    pub fn previous(&mut self) -> Option<(usize, String)> {
        self.current_index = self.current_index.saturating_sub(1);
        self.get(self.current_index)
    }

    //returns location at current index
    pub fn current(&self) -> Option<(usize, String)> {
        self.get(self.current_index)
    }

    //returns location from generic index
    pub fn get(&self, index: usize) -> Option<(usize, String)> {
        self.loc_list.get(index).map(|(o,s)| (*o, s.to_owned()))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut (usize, String)> {
        self.loc_list.get_mut(index)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn set_current_index(&mut self, index: usize) {
        self.current_index = std::cmp::min(index, self.loc_list.len().saturating_sub(1));
    }

    pub fn find_idx(&self, offset: usize) -> Option<usize> {
        self.loc_list.iter().enumerate().find_map(|(i, &(lo,_))| (lo == offset).then_some(i))
    }

    pub fn remove_current_location(&mut self) {
        if self.current_index < self.loc_list.len() {
            self.loc_list.remove(self.current_index);
            self.current_index = std::cmp::min(self.current_index, self.loc_list.len().saturating_sub(1));
        }
    }

    pub fn remove_location(&mut self, idx: usize) {
        if idx < self.loc_list.len() {
            self.loc_list.remove(idx);
            self.current_index = std::cmp::min(self.current_index, self.loc_list.len().saturating_sub(1));
        }
    }

    //add new location to list
    pub fn add_location(&mut self, offset: usize, string: String) {
        self.loc_list.push((offset, string));
    }

    //returns iterator
    pub fn iter(&self) -> std::slice::Iter<'_, (usize, String)> {
        self.loc_list.iter()
    }

    pub fn len(&self) -> usize {
        self.loc_list.len()
    }
}

impl FromIterator<(usize, String)> for LocationList {

    fn from_iter<I: IntoIterator<Item=(usize, String)>>(iter: I) -> Self {
        let mut ll = LocationList::new();
        iter.into_iter().for_each(|l| ll.loc_list.push(l));
        ll
    }
}
