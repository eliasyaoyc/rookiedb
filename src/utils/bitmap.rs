/// A simple fixed size bitmap.
#[derive(Clone)]
pub(crate) struct Bitmap {
    cap: u32,
    len: u32,
    bits: Vec<u64>,
}

pub(crate) struct BitmapIter<'a> {
    bitmap: &'a Bitmap,
    key: usize,
    value: u64,
}

impl Bitmap {
    pub(crate) fn new(cap: u32) -> Self {
        let size = match cap % 64 {
            0 => cap / 64,
            _ => cap / 64 + 1,
        };
        let bits = vec![0u64; size as usize];
        Self { cap, len: 0, bits }
    }

    /// Set the corresponding bit.
    pub(crate) fn set(&mut self, index: u32) -> bool {
        let (key, bit) = (key(index), bit(index));
        let old_w = self.bits[key];
        let new_w = old_w | 1 << bit;
        let inserted = (old_w ^ new_w) >> bit; // 1 or 0
        self.bits[key] = new_w;
        self.len += inserted as u32;
        inserted != 0
    }

    /// Clear the corresponding bit.
    pub(crate) fn clear(&mut self, index: u32) -> bool {
        let (key, bit) = (key(index), bit(index));
        let old_w = self.bits[key];
        let new_w = old_w & !(1 << bit);
        let removed = (old_w ^ new_w) >> bit;
        self.bits[key] = new_w;
        self.len -= removed as u32;
        removed != 0
    }

    /// Whether the specified bit is set.
    pub(crate) fn exist(&self, index: u32) -> bool {
        let (key, bit) = (key(index), bit(index));
        self.bits[key] & (1 << bit) != 0
    }

    /// Returns the number of unset bits.
    #[inline]
    pub(crate) fn free(&self) -> u32 {
        self.cap
            .checked_sub(self.len)
            .expect("The len does not exceed the capacity")
    }

    /// Returns the number of set bits.
    #[inline]
    #[allow(unused)]
    pub(crate) fn len(&self) -> u32 {
        self.len
    }

    /// Returns the total bits.
    #[inline]
    #[allow(unused)]
    pub(crate) fn cap(&self) -> u32 {
        self.cap
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.len == self.cap
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(crate) fn iter(&self) -> BitmapIter {
        BitmapIter::new(self)
    }
}

impl<'a> BitmapIter<'a> {
    fn new(bitmap: &'a Bitmap) -> Self {
        Self {
            bitmap,
            key: 0,
            value: bitmap.bits[0],
        }
    }
}

impl<'a> Iterator for BitmapIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.value == 0 {
                self.key += 1;
                if let Some(value) = self.bitmap.bits.get(self.key) {
                    self.value = *value;
                    continue;
                } else {
                    return None;
                }
            }

            let index = self.value.trailing_zeros() as usize;
            self.value &= self.value - 1;
            return Some((64 * self.key + index) as u32);
        }
    }
}

#[inline]
fn key(index: u32) -> usize {
    index as usize / 64
}

#[inline]
fn bit(index: u32) -> usize {
    index as usize % 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut bitmap = Bitmap::new(2048);
        
        for i in 0..2048 {
            bitmap.set(i);
        }

        let mut iter = bitmap.iter();
        while let Some(v) = iter.next() {
            println!("{}", v)
        }
    }
}
