use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum BitVecError {
    OutOfBounds(usize),
}

impl Display for BitVecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitVecError::OutOfBounds(len) => {
                write!(f, "Error: Given value is out of bounds: {}", len)
            }
        }
    }
}

impl Error for BitVecError {}

/// A struct for storing bits in u64 buffers. Will waste memory, as even a bit
/// vector of 1 bit takes up a whole u64, however, the vector won't need to be
/// reallocated as often.
///
/// # Fields
///
/// * 'buffer' - A vector of u64 values acting as a bit buffer.
/// * 'len' - A usize value containing the length of the bit vector.
/// * 'idx' - A usize value representing the current index in the buffer.
///         len % 64 the index within the current u64.
pub struct BitVector64 {
    pub buffer: Vec<u64>,
    pub len: usize,
    pub idx: usize,
}

impl BitVector64 {
    /// Creates a new empty BitVector64.
    ///
    /// # Returns
    ///
    /// A BitVector64 with default values.
    pub fn new() -> Self {
        Self {
            buffer: vec![0],
            len: 0,
            idx: 0,
        }
    }
    /// Pushes the len least significant bits of the given buffer to the
    /// BitVector64.
    ///
    /// # Arguments
    ///
    /// * 'buffer' - The bit buffer to push the bits from.
    /// * 'len' - The number of bits to push.
    ///
    /// # Returns
    ///
    /// A result containing either a unit value, or a BitVecError. Returns
    /// an error if the len is longer than 64, because it takes a u64 buffer.
    pub fn push_buffer(&mut self, buffer: u64, len: usize) -> Result<(), BitVecError> {
        // Check if len is out of bounds.
        if len > 64 {
            return Err(BitVecError::OutOfBounds(len));
        }
        let bit_idx = self.len % 64;
        let buf_idx = self.buffer.len() - 1;
        self.len += len;

        let shifted_buffer = buffer << (64 - len);

        if (bit_idx + len) >= 64 {
            self.buffer.push(0);
            self.buffer[buf_idx] |= shifted_buffer >> bit_idx;
            self.buffer[buf_idx + 1] = shifted_buffer << (64 - bit_idx);
        } else {
            self.buffer[buf_idx] |= shifted_buffer >> bit_idx;
        }

        Ok(())
    }
    /// Accepts a u8 representation of a bit, and pushes that bit to the
    /// vector. Reallocates the vector once the current vector is full,
    /// not once called while the current vector is full.
    ///
    /// # Arguments
    ///
    /// * 'bit' - A single u8 either 1 or 0 representing the bit to push.
    ///
    /// # Returns
    ///
    /// Returns nothing, or an error if bit is neither 0 or 1.
    #[inline]
    pub fn push_bit(&mut self, bit: u8) -> Result<(), BitVecError> {
        let bit_idx = self.len % 64;
        let buf_idx = self.buffer.len() - 1;

        if bit_idx >= 63 {
            self.buffer.push(0);
        }

        match bit {
            0 => {
                self.len += 1;
            }
            1 => {
                self.buffer[buf_idx] |= 1 << (63 - bit_idx);
                self.len += 1;
            }
            _ => {
                return Err(BitVecError::OutOfBounds(bit as usize));
            }
        }
        Ok(())
    }
    /// Builds a BitVector64 from a byte-aligned big-endian byte array.
    ///
    /// # Arguments
    ///
    /// * 'raw' - The byte array to build from.
    ///
    /// # Returns
    ///
    /// The constructed bit vector, with the first element being the least
    /// significant bit of the first byte, and the last element being the
    /// most significant bit of the last byte.
    pub fn from_be_bytes(raw: &[u8]) -> Self {
        let mut bit_vector = BitVector64::new();

        for byte in raw.iter().map(|x| x.reverse_bits()) {
            // Will never panic because push_buffer only returns an error if
            // len is more than 64.
            bit_vector.push_buffer(byte as u64, 8).unwrap();
        }

        bit_vector
    }
    /// Builds a BitVector64 from a byte-aligned little-endian byte array.
    ///
    /// # Arguments
    ///
    /// * 'raw' - The byte array to build from.
    ///
    /// # Returns
    ///
    /// The constructed bit vector, with the first element being the most
    /// significant bit of the first byte, and the last being the least
    /// significant bit from the last byte.
    pub fn from_le_bytes(raw: &[u8]) -> Self {
        let mut bit_vector = BitVector64::new();

        for byte in raw {
            bit_vector.push_buffer(byte.to_owned() as u64, 8).unwrap();
        }

        bit_vector
    }
    /// Removes the first bit in the stream and returns it. Only reallocates
    /// the vector once the current bit buffer is empty.
    ///
    /// # Returns
    ///
    /// An option containing either a u8 either 0 or 1, or None.
    pub fn pop_front(&mut self) -> Option<u8> {
        if self.len == 0 {
            None
        } else if self.idx < 64 {
            let value = ((self.buffer[0] & (1u64 << (63 - self.idx))) >> (63 - self.idx)) as u8;
            self.buffer[0] &= !(1u64 << (63 - self.idx));
            self.len -= 1;
            self.idx += 1;
            Some(value)
        } else {
            self.buffer.remove(0);
            self.idx = 0;
            self.pop_front()
        }
    }
}

impl Default for BitVector64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for BitVector64 {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            let byte_idx = self.idx / 64;
            let bit_idx = 63 - (self.idx % 64);
            let current_byte = self.buffer[byte_idx];
            let bit = (current_byte >> bit_idx) & 1;
            self.idx += 1;
            Some(bit as u8)
        } else {
            None
        }
    }
}
impl Display for BitVector64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for unsigned64 in self.buffer.clone() {
            writeln!(f, "{:064b}", unsigned64)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BitVector64;
    use std::fs;
    #[ignore]
    #[test]
    fn test_bitvector() {
        let mut new_bitvec = BitVector64::new();
        new_bitvec.push_buffer(0b1111, 4).unwrap();
        println!("{}", new_bitvec);
        new_bitvec.push_buffer(0b1111, 60).unwrap();
        println!("{}", new_bitvec);
        new_bitvec.push_buffer(0b1111, 4).unwrap();
        println!("{}", new_bitvec);

        let mut push_bit_vector = BitVector64::new();

        for _ in 0..128 {
            push_bit_vector.push_bit(1).unwrap();
            println!("{}", push_bit_vector);
        }
    }
    #[ignore]
    #[test]
    fn test_from_be() {
        let bytes = fs::read("./tests/data/block_type_0.gz").unwrap();

        let stream = BitVector64::from_be_bytes(&bytes);

        println!("{}", stream);
    }
    #[ignore]
    #[test]
    fn test_pop_front() {
        let mut bitvec = BitVector64::new();
        bitvec.push_buffer(0b1111, 8).unwrap();

        bitvec.push_buffer(0xfffffff, 64).unwrap();

        for _ in 0..128 {
            println!("{:?}", bitvec.pop_front());
        }
        println!("{}", bitvec);
    }
}
