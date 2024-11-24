//! Implementation of regular bitstreams as well as conversion
//! from a bitstream to bytes as utilized in RFC 1951.
//!

use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum BitError {
    InvalidPartialRange(u8, u8),
}

impl Display for BitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitError::InvalidPartialRange(x, y) => {
                write!(
                    f,
                    "Error: Invalid range for partial byte operation: {}, {}",
                    x, y
                )
            }
        }
    }
}

impl Error for BitError {}

/// Struct representing a bitstream/array of bits. Bits are
/// stored in a byte array, stored with a length to state how
/// many bits of that byte array are part of the bitstream. By
/// default bitstreams are big-endian but contain a function to
/// flip them to be little endian.
///
/// # Fields
///
/// * 'len' - A usize value representing the number of bits in
///         the bistream.
/// * 'idx' - A usize value representing the index of the current bit.
/// * 'bits' - A Vec<u8> containing bytes within which the bitstream
///         is stored, the vector will always be of length len/8 rounded
///         up. Bits are stored left to right, with the most significant
///         bit at the left.
///
/// # Methods
///
/// * 'new' - Generates a new empty bitstream with the specified endianness.
/// * 'push' - Takes in a 0 or 1 and pushes it to the right side of the bitstream.
/// * 'push_partial' - Takes in a byte and a start and end index and pushes the slice
///             of the given byte.
///
/// # Examples
///
/// '''
/// let stream = BitStream::new();
///
/// print!("{}", bitstream); // output: 0: 00000000
/// stream.push(1);
/// print!("{}", bitstream); // output: 0: 10000000
///
/// '''
pub struct BitStream {
    pub len: usize,
    pub idx: usize,
    pub bytes: Vec<u8>,
}

impl BitStream {
    /// Creates a new empty bitstream.
    ///
    /// # Returns
    ///
    /// The generated empty bitstream of the given endianness.
    ///
    /// # Examples
    ///
    /// '''
    /// let stream = BitStream::new();
    /// stream.push(0);
    /// '''
    pub fn new() -> Self {
        Self {
            len: 0,
            idx: 0,
            bytes: Vec::new(),
        }
    }
    /// Takes in a 0 or 1 and pushes it to the least significant
    /// end of the bistream, so for little endian bitstreams, the
    /// first bit of the first byte in the array will be altered
    ///
    /// # Arguments
    ///
    /// * 'self' - A mutable reference to self.
    /// * 'bit' - A u8 value that represent a 0 or a 1, if a non-binary value
    ///         is given, it will be normalized to a 1.
    pub fn push_bit(&mut self, bit: u8) {
        // Check if bit is a 0 or 1 and if not treat it as a 1 while
        // printing a warning.
        let normalized_bit: u8 = match bit {
            0 => 0,
            1 => 1,
            _ => {
                eprintln!(
                    "Warning: Non-binary value given to .push(), value has been corrected to a 1."
                );
                1
            }
        };

        let bit_index: u8 = (self.len % 8) as u8;

        // If the last byte in the byte array is not filled.
        if bit_index != 0 {
            // Take the last byte and shift out the unfilled bits
            // if bit_index = 3 and byte = 0xE0
            // 0b1110_0000 << 8 - bit_index (5)
            // 0b0000_0111 >>= 1;
            // bit-index += 1;
            // 0b0000_1111 >> 8 - bit_index (4)
            // final = 0b1111_0000
            let shift: u8 = 8 - bit_index - 1;
            let len = self.bytes.len() - 1;
            self.bytes[len] =
                ((self.bytes[self.bytes.len() - 1] >> shift) | normalized_bit) << shift;
            self.len += 1;
        } else {
            // If the last byte is full, define the working byte as
            // normalized_bit in shifted over 7 to make it the most
            // significant bit, then push it to the byte array before
            // incrementing len.
            let working_byte = normalized_bit << 7;
            self.bytes.push(working_byte);
            self.len += 1;
        }
    }
    /// Push a partial byte to the bit stream. Indexed from MSB to LSB.
    /// The ranges are inclusive.
    ///
    /// # Arguments
    ///
    /// * 'byte' - A u8 value containing the byte to take the slice from.
    /// * 'start' - A u8 value containing the start index.
    /// * 'end' - A u8 value containing the end index.
    ///
    /// # Examples
    ///
    /// '''
    /// let byte = 0b0000_1111;
    /// let stream = BitStream::new();
    ///
    /// stream.push_partial(byte, 4, 7);
    /// stream.push_partial(byte, 1, 4);
    ///
    /// assert_eq!(stream.bytes[0], 0b1111_0001);
    /// '''
    pub fn push_partial(&mut self, byte: u8, start: u8, end: u8) -> Result<(), BitError> {
        // Check for invalid ranges.
        if start > end || start >= 8 || end >= 8 {
            return Err(BitError::InvalidPartialRange(start, end));
        }

        let start = 7 - start;
        let end = 7 - end;

        for i in (end..=start).rev() {
            let bit = (byte >> i) & 1;
            self.push_bit(bit);
        }

        Ok(())
    }
    /// Takes in a u32 as a bit buffer and pushes the length least significant
    /// bits to the bitstream. Currently, it loops and pushes the singular
    /// which is likely a good bit slower than doing it in one operation, and
    /// concatenating the most recent n % (length * 2) bytes and bit-shifting
    /// them over by length and ORing with buffer, however, I don't feel like
    /// implementing that right now.
    ///
    /// TODO: Remove loop.
    ///
    /// # Arguments
    ///
    /// * 'buffer' - A u32 little-endian bit buffer storing the bits to push.
    /// * 'length' - The quantity of bits to push to the bitstream.
    ///
    pub fn push(&mut self, buffer: u32, length: u8) {
        for i in (0..length).rev() {
            self.push_bit(buffer.bit_index(i));
        }
    }
    /// RFC 1951 Section 3.1.1 describes the process of packing
    /// the bits into bytes as follows:
    ///     1. Data elements are packed into bytes in order of
    ///     increasing bit number within the byte, i.e., starting
    ///     with the least-significant bit of the byte.
    ///     2. Data elements other than Huffman codes are packed
    ///     starting with the least-significant bit of the data
    ///     element.
    ///     3. Huffman codes are packed starting with the most-
    ///    significant bit of the code.
    /// This function performs the inverse of this operation assuming
    /// the huffman codes have already been pushed in with the right orientation.
    /// Anticipates byte aligned data so unfilled bytes are truncated.
    pub fn to_rfc_bytes(&self) -> Vec<u8> {
        todo!();
    }
}

impl Default for BitStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for BitStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, byte) in self.bytes.iter().enumerate() {
            writeln!(f, "{}: {:08b}", i, byte)?;
        }
        Ok(())
    }
}

impl Iterator for BitStream {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            let byte_idx = self.idx / 8;
            let bit_idx = 7 - (self.idx % 8);
            let current_byte = self.bytes[byte_idx];
            let bit = (current_byte >> bit_idx) & 1;
            self.idx += 1;
            Some(bit)
        } else {
            None
        }
    }
}

/// Function to flip bytes from big endian to little endian using a constant lookup table.
pub fn reverse_byte(byte: u8) -> u8 {
    const LOOKUP: [u8; 256] = [
        0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0, 0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70,
        0xF0, 0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8, 0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8,
        0x78, 0xF8, 0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4, 0x14, 0x94, 0x54, 0xD4, 0x34,
        0xB4, 0x74, 0xF4, 0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC, 0x1C, 0x9C, 0x5C, 0xDC,
        0x3C, 0xBC, 0x7C, 0xFC, 0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2, 0x12, 0x92, 0x52,
        0xD2, 0x32, 0xB2, 0x72, 0xF2, 0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA, 0x1A, 0x9A,
        0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA, 0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6, 0x16,
        0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6, 0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE,
        0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE, 0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61,
        0xE1, 0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1, 0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9,
        0x69, 0xE9, 0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9, 0x05, 0x85, 0x45, 0xC5, 0x25,
        0xA5, 0x65, 0xE5, 0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5, 0x0D, 0x8D, 0x4D, 0xCD,
        0x2D, 0xAD, 0x6D, 0xED, 0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD, 0x03, 0x83, 0x43,
        0xC3, 0x23, 0xA3, 0x63, 0xE3, 0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3, 0x0B, 0x8B,
        0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB, 0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB, 0x07,
        0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7, 0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7,
        0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF, 0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F,
        0xFF,
    ];
    LOOKUP[byte as usize]
}

pub trait BitIndex {
    fn bit_index(&self, index: u8) -> u8;
}

impl BitIndex for u32 {
    fn bit_index(&self, index: u8) -> u8 {
        ((self >> (index)) & 1) as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::bitstream::{BitIndex, BitStream};

    #[test]
    fn test_bitstream() {
        let mut stream = BitStream::new();

        let bits: [u8; 8] = [1, 0, 1, 0, 1, 0, 1, 0];

        for bit in bits {
            stream.push_bit(bit);
        }

        let partial_byte: u8 = 0b0000_1111;
        stream.push_partial(partial_byte, 4, 7).unwrap();
        stream.push_partial(partial_byte, 1, 4).unwrap();

        let concatenated: u16 = ((stream.bytes[0] as u16) << 8) | stream.bytes[1] as u16;
        let concatenated_u32 = concatenated as u32;

        let to_push: u16 = 0b11_0011_0011;
        stream.push(to_push as u32, 10);

        let last_ten = ((stream.bytes[stream.bytes.len() - 2] as u16) << 8)
            | stream.bytes[stream.bytes.len() - 1] as u16;

        let bit_idx = stream.len % 8;

        assert_eq!(concatenated_u32.bit_index(9), 1);
        assert_eq!(concatenated, 0b10101010_11110001);
        assert_eq!(last_ten, to_push << (8 - bit_idx))
    }
}
