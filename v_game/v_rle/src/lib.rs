pub struct RLE<T>
    where T: Eq + PartialEq + Clone{
    raw: Vec<(usize, T)>
}

impl<T> RLE<T>
    where T: Eq + PartialEq + Clone{
    fn rle_compress<U>(data: U) -> Vec<(usize, T)>
    where U:Iterator<Item = T>{
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
                        count = 1;
                        current = Some(d.clone());
                    }
                },
                None =>{
                    current = Some(d.clone());
                }
            }
        }
        compressed
    }

    pub fn access(&self, index: usize) -> Result<T, RLEError>{
        let mut current = 0;
        for (num, data) in self.raw.iter(){
            let next = current + *num;
            if next >= index{
                return Ok(data.clone());
            }
            current = next;
        }
        Err(RLEError::OutOfRange)
    }

    pub fn set(&mut self, index: usize, item: &T) -> Result<(), RLEError>{
        let mut current = 0;
        let mut target_index = 0;
        for (i, (num, data)) in self.raw.iter().enumerate(){
            let next = current + *num;
            //It's somewhere *within* the current item
            if next >= index{
                target_index = i;
                break;
            }
            current = next;
        }

        let target_place = index - current;
        // place item
        if target_place == 0{
            self.raw[target_index].0 -= 1;
            self.raw.insert(target_index, (1, item.clone()));
        }else if target_place > 0{
            let second_half = self.raw[target_index].0 - target_place - 1;
            self.raw[target_index].0 -= second_half;
            self.raw.insert(target_index+1, (second_half, self.raw[target_index].1.clone()));
            self.raw.insert(target_index+1, (1, item.clone()));
        }

        // check if any of the three potentially changed items can be combined
        // then combine them
        if target_index < self.raw.len(){
            if self.raw[target_index + 1].1 == self.raw[target_index].1{
                self.raw[target_index].0 + self.raw[target_index+1].0;
                self.raw.remove(target_index+1);
            }
        }
        if target_index != 0{
            if self.raw[target_index - 1].1 == self.raw[target_index].1{
                self.raw[target_index - 1].0 + self.raw[target_index+1].0;
                self.raw.remove(target_index);
            }
        }


        Err(RLEError::OutOfRange)
    }
}

impl<T, U> From<U> for RLE<T>
    where T: Eq + PartialEq + Clone,
          U: Iterator<Item = T>{
    fn from(data: U) -> Self<T>{
        Self{
            raw: Self::rle_compress(data)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RLEError{
    OutOfRange,
}