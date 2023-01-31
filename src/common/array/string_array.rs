use bitvec::vec::BitVec;

use super::{iterator::ArrayIterator, Array, ArrayBuilder};

pub struct StringArray {
    /// The flattened data of string.
    data: Vec<u8>,

    /// Offsets of each string in the data flat array.
    offsets: Vec<usize>,

    /// The null bitmap of this array.
    bitmap: BitVec,
}

impl Array for StringArray {
    type Builder = StringArrayBuilder;
    type OwnedItem = String;

    fn get(&self, idx: usize) -> Option<&String> {
        if self.bitmap[idx] {
            let range = self.offsets[idx]..self.offsets[idx + 1];
            // Some(unsafe { std::str::from_utf8_unchecked(&self.data[range]) })
            None
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.bitmap.len()
    }

    fn iter(&self) -> ArrayIterator<Self> {
        ArrayIterator::new(self)
    }
}

pub struct StringArrayBuilder {
    /// The flattened data of string.
    data: Vec<u8>,

    /// Offsets of each string in the data flat array.
    offsets: Vec<usize>,

    /// The null bitmap of this array.
    bitmap: BitVec,
}

impl ArrayBuilder for StringArrayBuilder {
    type Array = StringArray;

    fn with_capacity(capacity: usize) -> Self {
        let mut offsets = Vec::with_capacity(capacity);
        offsets.push(0);
        Self {
            data: Vec::with_capacity(capacity),
            bitmap: BitVec::with_capacity(capacity),
            offsets,
        }
    }

    fn push(&mut self, value: Option<String>) {
        match value {
            Some(v) => {
                self.data.extend(v.as_bytes());
                self.offsets.push(self.data.len());
                self.bitmap.push(true);
            }
            None => {
                self.offsets.push(self.data.len());
                self.bitmap.push(false);
            }
        }
    }

    fn finish(self) -> Self::Array {
        StringArray {
            data: self.data,
            offsets: self.offsets,
            bitmap: self.bitmap,
        }
    }
}
