use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{
    bits::BitVector64,
    prefix::{
        PrefixTree, DISTANCE_BASE, DISTANCE_CODES, DISTANCE_EXTRA_BITS, FIXED_CODE_LENGTHS,
        LENGTH_BASE, LENGTH_CODES, LENGTH_EXTRA_BITS,
    },
};

#[derive(Debug)]
pub enum DeflateError {
    InvalidBlockError(&'static str),
    InvalidSymbolError(usize, &'static str),
}

impl Display for DeflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeflateError::InvalidBlockError(s) => {
                write!(f, "InvalidBlock Error: {}", s)
            }
            DeflateError::InvalidSymbolError(v, r) => {
                write!(f, "InvalidSymbolError cause by symbol: {}, {}", v, r)
            }
        }
    }
}

impl Error for DeflateError {}

pub struct DeflateData {
    compressed: Vec<u8>,
    decompressed: Vec<u8>,
    bitstream: BitVector64,
    finished: bool,
}

impl DeflateData {
    pub fn build(compressed: &[u8]) -> Self {
        let bitstream = BitVector64::from_be_bytes(compressed);
        Self {
            compressed: compressed.to_vec(),
            decompressed: Vec::new(),
            bitstream,
            finished: false,
        }
    }
    pub fn decompress(&mut self) -> Result<(), DeflateError> {
        while !self.finished {
            // Initialize header.
            let mut header: [u8; 3] = [0; 3];

            // Iterate through header, popping the first 3 items from the
            // bitstream and adding them to header.
            for header_bit in header.iter_mut() {
                if let Some(b) = self.bitstream.next() {
                    *header_bit = b;
                } else {
                    return Err(DeflateError::InvalidBlockError(
                        "Block ran out of bits before a header was specified.",
                    ));
                }
            }

            self.finished = matches!(header[0], 1);

            // Main decompression loop.
            match (header[1], header[2]) {
                (0, 0) => {
                    self.block_type_0()?;
                    println!("{}", String::from_utf8_lossy(&self.decompressed));
                }
                (1, 0) => {
                    self.block_type_1()?;
                    println!("BTYPE 1");
                }
                (0, 1) => {
                    println!("BTYPE 2");
                }
                _ => return Err(DeflateError::InvalidBlockError("Invalid BTYPE.")),
            }
        }
        Ok(())
    }
    fn block_type_0(&mut self) -> Result<(), DeflateError> {
        let len = self
            .bitstream
            .by_ref()
            .skip(5)
            .take(16)
            .fold(0u16, |acc, bit| (acc << 1) | (bit as u16))
            .reverse_bits();

        // Take the subsequent 16 bits as a u16.
        let nlen = self
            .bitstream
            .by_ref()
            .take(16)
            .fold(0u16, |acc, bit| (acc << 1) | (bit as u16))
            .reverse_bits();

        if len != !nlen {
            return Err(DeflateError::InvalidBlockError(
                "BTYPE is 0, but NLEN is not the bitwise complement to LEN.",
            ));
        }

        // Figure out what byte the current index is in.
        let byte_idx = self.bitstream.idx / 8;

        self.compressed[byte_idx..len as usize + byte_idx]
            .iter()
            .for_each(|x| self.decompressed.push(*x));

        Ok(())
    }
    fn block_type_1(&mut self) -> Result<(), DeflateError> {
        let mut prefix_tree = PrefixTree::from_lengths(&FIXED_CODE_LENGTHS);

        // Generate HashMaps from the constant value tables in prefix.
        let mut length_extra = HashMap::new();
        let mut length_base = HashMap::new();

        let mut distance_extra = HashMap::new();
        let mut distance_base = HashMap::new();

        for (idx, length) in LENGTH_CODES.iter().enumerate() {
            length_extra.insert(length.to_owned(), LENGTH_EXTRA_BITS[idx]);
            length_base.insert(length.to_owned(), LENGTH_BASE[idx]);
        }

        for (idx, distance) in DISTANCE_CODES.iter().enumerate() {
            distance_extra.insert(distance.to_owned(), DISTANCE_EXTRA_BITS[idx]);
            distance_base.insert(distance.to_owned(), DISTANCE_BASE[idx]);
        }

        let mut output = Vec::new();

        // Iterate through the bitstream.
        while let Some(bit) = self.bitstream.by_ref().next() {
            // Walk the tree and if there is a value, take it and continue.
            if let Some(value) = prefix_tree.walk(bit) {
                // If the value less than 256, it is a literal and should be
                // pushed unaltered to the output stream.
                if value < 256 {
                    output.push(value);
                // If it is in the range from 257..285 it is a length code.
                } else if let 257..=285 = value {
                    // Get the base and number of extra bits.
                    // Unwrap is called because the only circumstance this
                    // is a call to a HashMap made from a const table, so if
                    // it returns None something else has gone horribly wrong.
                    let mut _length = *length_base.get(&value).unwrap();
                    let len_extra = *length_extra.get(&value).unwrap();
                    // If length has extra bits, iterate through them, and add
                    // the value to the base length.
                    if len_extra > 0 {
                        let mut additional_length = 0;
                        for _ in 0..len_extra {
                            if let Some(bit) = self.bitstream.by_ref().next() {
                                additional_length = (additional_length << 1) & bit;
                            }
                        }
                        _length += additional_length as u16;
                    }

                    // After every length code is a 5 bit distance code.
                    let mut _distance: usize = 0;
                    for _ in 0..5 {
                        if let Some(bit) = self.bitstream.by_ref().next() {
                            _distance = (_distance << 1) & bit as usize;
                        }
                    }

                    let (dist_extra, dist_base) = match (
                        distance_extra.get(&_distance),
                        distance_base.get(&_distance),
                    ) {
                        (Some(extra), Some(base)) => (*extra, *base),
                        (_, _) => {
                            return Err(DeflateError::InvalidSymbolError(_distance, "Failed to get distance code base and extra bits, invalid distance code symbol."));
                        }
                    };

                    if dist_extra > 0 {
                        let mut additional_distance = 0;
                        for _ in 0..dist_extra {
                            if let Some(bit) = self.bitstream.by_ref().next() {
                                additional_distance = (additional_distance << 1) & bit;
                            }
                        }
                        _distance = (dist_base + (additional_distance as u16)) as usize;
                    } else {
                        _distance = dist_base as usize;
                    }

                    let start_idx = output.len() - _distance;
                    let end_idx = start_idx + _length as usize;

                    for idx in start_idx..end_idx {
                        output.push(output[idx]);
                    }
                } else if value == 256 {
                    break;
                }
            }
        }
        output
            .iter()
            .map(|x| *x as u8)
            .for_each(|byte| self.decompressed.push(byte));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::gzip::GzipFile;

    #[test]
    fn decompress_test() {
        let mut file = GzipFile::from_path("./tests/data/block_type_1_abc.gz").unwrap();

        file.deflate.decompress().unwrap();
        println!("{}", String::from_utf8_lossy(&file.deflate.decompressed));
    }
}
