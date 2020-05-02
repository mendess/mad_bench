use std::mem::size_of;

/// This vector represents an array of packed numbers
///
/// If `register_size` is 5 then each number takes 5 bits of storage.
///
pub struct BitArray {
    register_size: u8,
    elems: Box<[u8]>,
    capacity: usize,
}

const WORD_SIZE: usize = size_of::<u8>() * 8;

impl BitArray {
    pub fn new(register_size: u8, capacity: usize) -> Self {
        assert!(
            (register_size as usize) < WORD_SIZE,
            "Register size has to be less than {}",
            WORD_SIZE
        );
        let register_size_ = register_size as usize;
        let slice_len = ((register_size_ * capacity) + (WORD_SIZE - 1)) / WORD_SIZE;
        Self {
            register_size,
            elems: vec![0; slice_len].into(),
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.capacity
    }

    pub fn set(&mut self, index: usize, value: u8) {
        // [xxxxxyyy|yyzzzzzw|wwww0000]
        //       s     e^^^^
        // bit_index + register_size > 8
        let start_bit = (index * self.register_size as usize) % WORD_SIZE;
        let end_bit = start_bit + self.register_size as usize;
        let slice_index = (index * self.register_size as usize) / WORD_SIZE;
        if end_bit > WORD_SIZE {
            self.elems[slice_index] |= value >> (end_bit - WORD_SIZE);
            self.elems[slice_index + 1] |= value << ((2 * WORD_SIZE) - end_bit);
        } else {
            let value = value << (WORD_SIZE - end_bit);
            self.elems[slice_index] |= value;
        }
    }

    pub fn get(&self, index: usize) -> u8 {
        // [xxxxxyyy|yyzzzzzw|wwww0000]
        //       s     e^^^^
        // bit_index + register_size > 8
        let start_bit = (index * self.register_size as usize) % WORD_SIZE;
        let end_bit = start_bit + self.register_size as usize;
        let slice_index = (index * self.register_size as usize) / WORD_SIZE;
        if end_bit > WORD_SIZE {
            let v0 = self.elems[slice_index];
            let v1 = self.elems[slice_index + 1];
            (u16::from_be_bytes([v0, v1]) >> (2 * WORD_SIZE - end_bit)) as u8
                & init_right_mask(self.register_size as usize)
        } else {
            let value = self.elems[slice_index];
            let mask = ((1u16 << self.register_size) - 1) as u8;
            (value >> (WORD_SIZE - end_bit)) & mask
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            slice: &*self.elems,
            register_size: self.register_size,
            mask: init_left_mask(self.register_size as usize),
            count: self.capacity,
        }
    }

    pub fn iter2(&self) -> Iter2<'_> {
        Iter2 {
            slice: self,
            index: 0,
            cap: self.capacity,
        }
    }

    pub fn iter3(&self) -> impl Iterator<Item = u8> + '_ {
        (0..self.capacity).map(move |i| self.get(i))
    }
}

const fn init_left_mask(r_size: usize) -> u8 {
    ((init_right_mask(r_size) as u16) << (WORD_SIZE - r_size)) as u8
}

const fn init_right_mask(r_size: usize) -> u8 {
    ((1u16 << r_size) - 1) as u8
}

pub struct Iter<'a> {
    slice: &'a [u8],
    register_size: u8,
    mask: u8,
    count: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let split = self.mask.count_ones() != self.register_size as u32;
        let r = match self.slice {
            [fst, snd, ..] if split => {
                let fst_snd = u16::from_be_bytes([*fst, *snd]);
                let n_past_boundary = self.register_size as u32 - ((!self.mask).trailing_zeros());
                let off_from_base = WORD_SIZE as u32 - n_past_boundary;
                self.mask = init_left_mask(self.register_size as usize);
                let r = (fst_snd >> off_from_base) & ((1u16 << self.register_size) - 1);
                self.mask >>= n_past_boundary;
                self.slice = &self.slice[1..];
                Some(r as u8)
            }
            [fst, ..] if !split => {
                let r = (fst & self.mask) >> self.mask.trailing_zeros();
                self.mask >>= self.register_size;
                if self.mask == 0 {
                    self.mask = init_left_mask(self.register_size as usize);
                    self.slice = &self.slice[1..];
                }
                Some(r)
            }
            [] => None,
            _ => unreachable!(),
        };
        self.count = match self.count.checked_sub(1) {
            Some(0) | None => {
                self.slice = &[];
                0
            }
            Some(c) => c,
        };
        r
    }
}

pub struct Iter2<'a> {
    slice: &'a BitArray,
    index: usize,
    cap: usize,
}

impl Iterator for Iter2<'_> {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.cap {
            None
        } else {
            let e = self.slice.get(self.index);
            self.index += 1;
            Some(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set() {
        for i in 1..128 {
            for w in 1..8 {
                eprintln!("Creating array with w = {} and cap = {}", w, i);
                let num_max = 1 << w;
                let mut v = BitArray::new(w, i);
                for n in 0..i {
                    v.set(n, (n as u8) % num_max);
                }
                for n in 0..i {
                    assert_eq!(n as u8 % num_max, v.get(n), "Indexing: {}", n);
                }
            }
        }
    }

    #[test]
    fn iter() {
        for i in 1..128 {
            for w in 1..8 {
                eprintln!("Creating array with w = {} and cap = {}", w, i);
                let num_max = 1 << w;
                let mut v = BitArray::new(w, i);
                for n in 0..i {
                    v.set(n, (n as u8) % num_max);
                }
                let vv = v.iter().collect::<Vec<_>>();
                assert!(
                    vv.iter().copied().eq((0..(i as u8)).map(|i| i % num_max)),
                    "Expected '{:?}' got '{:?}",
                    (0..(i as u8)).map(|i| i % num_max).collect::<Vec<_>>(),
                    vv
                )
            }
        }
    }

    #[test]
    fn len() {
        for i in 1..16 {
            for w in 1..8 {
                eprintln!("Creating array with r_size = {} and cap = {}", w, i);
                let v = BitArray::new(w, i);
                assert_eq!(v.len(), i);
                v.get(i - 1);
            }
        }
    }
}
