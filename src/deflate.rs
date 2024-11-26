use crate::{
    bitstream::{self, BitStream},
    prefix::{
        self, Code, PrefixTree, DISTANCE_BASE, DISTANCE_CODES, DISTANCE_EXTRA_BITS,
        FIXED_CODE_LENGTHS, LENGTH_BASE, LENGTH_CODES, LENGTH_EXTRA_BITS,
    },
};
use std::{collections::HashMap, error::Error, fmt::Display};

#[derive(Debug)]
pub enum DeflateError {
    InvalidBlock(&'static str),
}

impl Display for DeflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeflateError::InvalidBlock(s) => {
                write!(f, "InvalidBlock Error: {}", s)
            }
        }
    }
}

impl Error for DeflateError {}

pub struct DeflateBlock {
    pub bfinal: bool,
    pub btype: u8,
    pub raw: Vec<u8>,
}

impl DeflateBlock {
    pub fn build(bytes: &[u8]) -> Result<Self, DeflateError> {
        let bfinal = bytes[0] & 0b0000_0001 == 0b0000_0001;
        let btype = match bytes[0] {
            v if (v & 0b0000_0110 == 0) => 0,
            v if (v & 0b0000_0010 != 0) => 1,
            v if (v & 0b0000_0100 != 0) => 2,
            _ => {
                return Err(DeflateError::InvalidBlock(
                    "Invalid block type while building DeflateBlock",
                ));
            }
        };

        Ok(Self {
            bfinal,
            btype,
            raw: bytes.to_vec(),
        })
    }
    pub fn decompress(&self) -> Result<Vec<u8>, DeflateError> {
        let decompressed = match self.btype {
            0 => self.uncompressed(),
            1 => self.fixed_codes(),
            2 => self.dynamic_codes(),
            _ => {
                return Err(DeflateError::InvalidBlock(
                    "Invalid block type while decompressing.",
                ));
            }
        };
        Ok(decompressed)
    }
    fn uncompressed(&self) -> Vec<u8> {
        let len = u16::from_le_bytes([self.raw[1], self.raw[2]]);

        let output = self
            .raw
            .iter()
            .skip(5)
            .map(|x| x.to_owned())
            .take(len as usize)
            .collect::<Vec<_>>();

        output
    }
    fn fixed_codes(&self) -> Vec<u8> {
        let mut prefix_tree = PrefixTree::from_lengths(&FIXED_CODE_LENGTHS);

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

        let stream = BitStream::from_be(&self.raw).skip(3).collect::<Vec<_>>();

        let mut output: Vec<usize> = Vec::new();

        let mut idx = 0;

        // Avoid potential infinite recursion if EOB code is not found by the
        // end of the bitstream.
        while idx <= stream.len() {
            if let Some(value) = prefix_tree.walk(stream[idx]) {
                if value < 256 {
                    output.push(value);
                } else if value == 256 {
                    break;
                } else if let 257..=285 = value {
                    let mut length = length_base.get(&value).unwrap().to_owned();
                    if length_extra.get(&value).unwrap() > &0 {
                        let mut len_extra = 0;
                        for _ in 0..length_extra.get(&value).unwrap().to_owned() as isize {
                            idx += 1;
                            len_extra = (len_extra << 1) & stream[idx];
                        }
                        length += len_extra as u16;
                    }
                    println!("{}", length);
                    let mut dist = 0usize;
                    for _ in 0..5 {
                        idx += 1;
                        dist = (dist << 1) & stream[idx] as usize;
                    }
                    if distance_extra.get(&dist).unwrap() > &0 {
                        let mut extra = 0;
                        for _ in 0..distance_extra.get(&dist).unwrap().to_owned() as isize {
                            idx += 1;
                            extra = (extra << 1) & stream[idx];
                        }
                        dist = (distance_base.get(&dist).unwrap() + (extra as u16)) as usize;
                    } else {
                        dist = distance_base.get(&dist).unwrap().to_owned() as usize;
                    }
                    println!("{}", dist);

                    let start_idx = output.len() - dist;
                    let end_idx = start_idx + length as usize;

                    for idx in start_idx..end_idx {
                        output.push(output[idx]);
                    }
                }
            }
            idx += 1;
        }
        output.iter().map(|x| *x as u8).collect::<Vec<_>>()
    }
    fn dynamic_codes(&self) -> Vec<u8> {
        println!("DYNAMIC CODES");
        todo!()
    }
}
