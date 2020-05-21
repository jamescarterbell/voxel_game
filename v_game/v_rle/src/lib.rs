use std::thread::current;

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RLEError{
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    #[test]
    fn get_set() {
        let length = 100000;
        let mut longer = 0;
        let mut ratio = 0.0;
        let test_num = 100;
        for tests in 0..test_num {
            let mut rle = RLE::from(Some(random::<u32>()).into_iter().cycle().take(length));
            for i in 0..100000 {
                let value = random::<u32>() % 400;
                let coord = length / (400);
                let place = coord * value as usize + (random::<usize>() % 3);
                rle.set(place, &value);
                assert_eq!(rle.get(place).unwrap(), value);
            }
            let temp_ratio = ((rle.compressed_len() as f32) * (4.0 + 4.0)) / ((rle.raw_length as f32) * 4.0);
            ratio += temp_ratio;
            if temp_ratio > 1.0{
                longer += 1;
            }
        }
        ratio /= test_num as f32;
    }
}