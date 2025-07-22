use pyo3::{Bound, PyAny};
use std::marker::PhantomData;

pub enum Part<'py> {
    Interpolation(Bound<'py, PyAny>),
    Bytes(&'py [u8]),
}

pub enum PartByte<'a, 'py> {
    Interpolation(&'a Bound<'py, PyAny>),
    Bytes(&'a u8),
}

pub struct PartByteIterator<'a, 'py, I> {
    parts: I,
    current_bytes: Option<std::slice::Iter<'a, u8>>,
    _phantom: PhantomData<&'py ()>,
}

impl<'a, 'py, I> Iterator for PartByteIterator<'a, 'py, I>
where
    I: Iterator<Item = &'a Part<'py>>,
    'py: 'a,
{
    type Item = PartByte<'a, 'py>;

    fn next(&mut self) -> Option<Self::Item> {
        // First, check if we're currently iterating through bytes
        if let Some(ref mut byte_iter) = self.current_bytes {
            if let Some(byte) = byte_iter.next() {
                return Some(PartByte::Bytes(byte));
            } else {
                self.current_bytes = None;
            }
        }

        // Get the next part
        match self.parts.next() {
            Some(Part::Interpolation(bound)) => Some(PartByte::Interpolation(bound)),
            Some(Part::Bytes(bytes)) => {
                let mut byte_iter = bytes.iter();
                if let Some(first_byte) = byte_iter.next() {
                    self.current_bytes = Some(byte_iter);
                    Some(PartByte::Bytes(first_byte))
                } else {
                    // Empty byte slice, continue to next part
                    self.next()
                }
            }
            None => None,
        }
    }
}

pub fn parts_to_part_bytes<'a, 'py, I>(parts: I) -> PartByteIterator<'a, 'py, I::IntoIter>
where
    I: IntoIterator<Item = &'a Part<'py>>,
    'py: 'a,
{
    PartByteIterator {
        parts: parts.into_iter(),
        current_bytes: None,
        _phantom: PhantomData,
    }
}
