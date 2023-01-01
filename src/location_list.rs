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
        None
    }

    //returns previous location
    pub fn previous(&mut self) -> Option<(usize, String)> {
        if let Some(v) = self.get(self.current_index - 1) {
            self.current_index -= 1;
            return Some(v);
        } 
        None
    }

    //returns location at current index
    pub fn current(&self) -> Option<(usize, String)> {
        self.get(self.current_index)
    }

    //returns location from generic index
    pub fn get(&self, index: usize) -> Option<(usize, String)> {
        self.loc_list.get(index).map(|(o,s)| (*o, s.to_owned()))
    }
    
    pub fn current_index(&self) -> usize {
        self.current_index
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
