use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct Location {
    pub name: String,
    pub offset: usize,
    pub size: usize
}

#[derive(Clone)]
pub struct LocationList {
    current_index: usize,
    loc_list: Vec<Location>
}

impl LocationList {

    pub fn new() -> LocationList {
        LocationList {
            current_index: 0,
            loc_list: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.loc_list.is_empty()
    }

    //returns next location
    pub fn next(&mut self) -> Option<&Location> {
        if self.get(self.current_index + 1).is_some() {
            self.current_index += 1;
        }
        self.current()
    }

    //returns previous location
    pub fn previous(&mut self) -> Option<&Location> {
        self.current_index = self.current_index.saturating_sub(1);
        self.get(self.current_index)
    }

    //returns location at current index
    pub fn current(&self) -> Option<&Location> {
        self.get(self.current_index)
    }

    //returns location from generic index
    pub fn get(&self, index: usize) -> Option<&Location> {
        self.loc_list.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Location> {
        self.loc_list.get_mut(index)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn set_current_index(&mut self, index: usize) {
        self.current_index = std::cmp::min(index, self.loc_list.len().saturating_sub(1));
    }

    pub fn find_idx(&self, offset: usize) -> Option<usize> {
        self.loc_list.iter().enumerate().find_map(|(i, loc)| (loc.offset <= offset && (loc.offset + loc.size.saturating_sub(1)) >= offset).then_some(i))
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
    pub fn add_location(&mut self, location: Location) {
        self.loc_list.push(location);
    }

    pub fn len(&self) -> usize {
        self.loc_list.len()
    }
}

impl FromIterator<Location> for LocationList {

    fn from_iter<I: IntoIterator<Item=Location>>(iter: I) -> Self {
        let mut ll = LocationList::new();
        iter.into_iter().for_each(|l| ll.add_location(l));
        ll
    }
}

impl FromIterator<(String, usize)> for LocationList {

    fn from_iter<I: IntoIterator<Item=(String, usize)>>(iter: I) -> Self {
        let mut ll = LocationList::new();
        iter.into_iter().for_each(|t| ll.add_location(Location{name: t.0, offset: t.1, size: 0}));
        ll
    }
}

impl IntoIterator for LocationList {
    type Item = Location;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.loc_list.into_iter()
    }
}

impl<'a> IntoIterator for &'a LocationList {
    type Item = &'a Location;
    type IntoIter = core::slice::Iter<'a, Location>;

    fn into_iter(self) -> Self::IntoIter {
        self.loc_list.iter()
    }
}

impl<'a> IntoIterator for &'a mut LocationList {
    type Item = &'a mut Location;
    type IntoIter = core::slice::IterMut<'a, Location>;

    fn into_iter(self) -> Self::IntoIter {
        self.loc_list.iter_mut()
    }
}

impl Extend<Location> for LocationList {

    fn extend<T: IntoIterator<Item=Location>>(&mut self, iter: T){
        for location in iter {
            self.add_location(location);
        }
    }
}

impl Index<usize> for LocationList {
    type Output = Location;

    fn index(&self, index: usize) -> &Self::Output {
        self.loc_list.index(index)
    }
}

impl IndexMut<usize> for LocationList {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.loc_list.index_mut(index)
    }
}
