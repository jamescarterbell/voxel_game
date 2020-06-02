use std::thread::current;
use std::ops::Range;

/// A simple run length encoded compressed vector.
/// All items stored/returned by value meaning they must be cloneable.
pub struct RLE<T>
    where T: Eq + PartialEq + Clone{
    raw: Vec<(usize, T)>,
    pub raw_length: usize,
}

impl<T> RLE<T>
    where T: Eq + PartialEq + Clone{
    fn rle_compress<U>(data: U) -> (Vec<(usize, T)>, usize)
    where U:Iterator<Item = T>{
        let mut size = 0;
        let mut count = 0;
        let mut current: Option<T> = None;
        let mut compressed = Vec::new();
        for d in data{
            match current{
                Some(cur) => {
                    if cur == d{
                        count += 1;
                        current = Some(cur.clone());
                    }
                    else{
                        compressed.push((count, cur.clone()));
                        size += count;
                        count = 1;
                        current = Some(d.clone());
                    }
                },
                None =>{
                    current = Some(d.clone());
                    count += 1;
                }
            }
        }
        compressed.push((count, current.unwrap()));
        size += count;
        (compressed, size)
    }

    /// Get a value at a given index
    pub fn get(&self, index: usize) -> Result<T, RLEError>{
        if index >= self.raw_length{
            return Err(RLEError::OutOfRange);
        }
        let mut current = 0;
        for (num, data) in self.raw.iter(){
            let next = current + *num;
            if next > index {
                return Ok(data.clone());
            }
            current = next;
        }
        Ok(self.raw.last().unwrap().1.clone())
    }

    /// Set a value at the given index, value set via cloning
    pub fn set(&mut self, index: usize, item: &T) -> Result<(), RLEError>{
        if index >= self.raw_length{
            return Err(RLEError::OutOfRange);
        }
        let mut current = 0;
        let mut target_index = 0;
        for (i, (num, data)) in self.raw.iter().enumerate(){
            let next = current + *num;
            //It's somewhere *within* the current item
            if next > index{
                target_index = i;
                break;
            }
            current = next;
        }

        let target_place = index - current;

        let second_half = self.raw[target_index].0 - target_place - 1;

        if self.raw[target_index].1 != *item {
            self.raw[target_index].0 = target_place;
            if second_half > 0 {
                self.raw.insert(target_index + 1, (second_half, self.raw[target_index].1.clone()));
            }

            self.raw.insert(target_index + 1, (1, item.clone()));
            if self.raw[target_index].0 == 0 {
                self.raw.remove(target_index);
            }
        }

        if target_index + 1 < self.raw.len(){
            if self.raw[target_index + 1].1 == self.raw[target_index].1{
                self.raw[target_index].0 += self.raw[target_index + 1].0;
                self.raw.remove(target_index + 1);
            }
        }

        if target_index > 1 && target_index < self.raw.len(){
            if self.raw[target_index - 1].1 == self.raw[target_index].1{
                self.raw[target_index].0 += self.raw[target_index - 1].0;
                self.raw.remove(target_index - 1);
            }
        }
        Ok(())
    }

    /// Get a range over a group of values, returns as a vector and not an iterator
    /// meaning that this get's an uncompressed range
    pub fn get_range(&self, index: Range<usize>) -> Result<Vec<T>, RLEError>{
        if index.start >= self.raw_length || index.end > self.raw_length{
            return Err(RLEError::OutOfRange);
        }
        let mut vec = Vec::new();
        for item in self.iter().skip(index.start).take(index.end - index.start){
            vec.push(item.clone());
        }
        Ok(vec)
    }

    /// Set a range to a singular value
    pub fn set_range_singular(&mut self, item: T, index: Range<usize>) -> Result<(), RLEError>{
        if index.start >= self.raw_length || index.end > self.raw_length{
            return Err(RLEError::OutOfRange);
        }
        let mut current_start = 0;
        let mut start_index = 0;
        // Find the beginning placement of the range
        for (i, (num, data)) in self.raw.iter().enumerate(){
            let next = current_start + *num;
            //It's somewhere *within* the current item
            if next > index.start{
                start_index = i;
                break;
            }
            current_start = next;
        }
        // Find the end placement of the range
        let mut end_index = 0;
        let mut current_end = 0;
        for (i, (num, data)) in self.raw.iter().enumerate(){
            let next = current_end + *num;
            //It's somewhere *within* the current item
            if next > index.end{
                end_index = i;
                break;
            }
            current_end = next;
        }

        let target_start_place = index.start - current_start;
        let target_end_place = index.end - current_end;

        let second_half_end = if target_end_place > 0 {self.raw[end_index].0 - target_end_place} else {0} ;

        let start_item = self.raw[start_index].1.clone();
        let end_item = self.raw[end_index].1.clone();

        for i in (start_index..end_index + 1).rev(){
            self.raw.remove(i);
        }

        self.raw.insert(start_index, (second_half_end, end_item));
        self.raw.insert(start_index, (index.end - index.start, item));
        self.raw.insert(start_index, (target_start_place, start_item));


        if start_index + 2 < self.raw.len() && self.raw[start_index + 2].0 == 0{
            self.raw.remove(start_index + 2);
        }
        if start_index + 1 < self.raw.len() && self.raw[start_index + 1].0 == 0{
            self.raw.remove(start_index + 1);
        }
        if self.raw[start_index].0 == 0{
            self.raw.remove(start_index);
        }

        if start_index + 2 < self.raw.len() && self.raw[start_index + 2].1 == self.raw[start_index + 1].1{
            self.raw[start_index + 1].0 += self.raw[start_index + 2].0;
            self.raw.remove(start_index + 2);
        }

        if start_index + 1 < self.raw.len() && self.raw[start_index + 1].1 == self.raw[start_index].1{
            self.raw[start_index].0 += self.raw[start_index + 1].0;
            self.raw.remove(start_index + 1);
        }

        if start_index < self.raw.len() && start_index > 0 && self.raw[start_index - 1].1 == self.raw[start_index].1{
            self.raw[start_index - 1].0 += self.raw[start_index].0;
            self.raw.remove(start_index);
        }

        Ok(())
    }

    pub fn compressed_len(&self) -> usize{
        self.raw.len()
    }

    pub fn iter(&self) -> RLEIterator<T>{
        RLEIterator{
            rle: self,
            index: 0,
            current: 0,
        }
    }
}

impl<T, U> From<U> for RLE<T>
    where T: Eq + PartialEq + Clone,
          U: Iterator<Item = T>{
    fn from(data: U) -> Self{
        let (v, s) = Self::rle_compress(data);
        Self{
            raw: v,
            raw_length: s,
        }
    }
}

pub struct RLEIterator<'a, T>
    where T: Eq + PartialEq + Clone{
    rle: &'a RLE<T>,
    index: usize,
    current: usize,
}

impl<'a, T> Iterator for RLEIterator<'a, T>
    where T: Eq + PartialEq + Clone{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.rle.raw.len(){
            return None;
        }
        if self.current >= self.rle.raw[self.index].0{
            let item = self.rle.raw[self.index].1.clone();
            self.current = 0;
            self.index += 1;
            return Some(item);
        }
        None
    }
}

unsafe impl<T> Send for RLE<T>
    where T: Eq + PartialEq + Clone + Send{}

unsafe impl<T> Sync for RLE<T>
    where T: Eq + PartialEq + Clone + Send{}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RLEError{
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use std::iter::{repeat_with, repeat};

    #[test]
    fn get_set() {
        let mut r: RLE<usize> = RLE::from(repeat(0).take(10000));
        for (number, item) in r.raw.iter(){
            println!("number: {}\t item: {}", number, item);
        }
        println!("=========================");
        r.set_range_singular(1, 0..20);
        for (number, item) in r.raw.iter(){
            println!("number: {}\t item: {}", number, item);
        }
        println!("=========================");
        r.set_range_singular(1, 20..25);
        for (number, item) in r.raw.iter(){
            println!("number: {}\t item: {}", number, item);
        }
    }
}