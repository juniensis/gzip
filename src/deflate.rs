use crate::{
    bitstream::{self, BitStream},
    prefix,
};
use std::{error::Error, fmt::Display};

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
        todo!()
    }
    fn dynamic_codes(&self) -> Vec<u8> {
        todo!()
    }
}
