//! Gzip encoding and decoding.
use std::{
    error::Error,
    fmt::Display,
    fs, io,
    path::Path,
    str::{from_utf8, from_utf8_unchecked},
};

/// A struct containing the individual parts of a GZIP header.
///
/// # Fields
///
/// * 'cm' - A single u8 value representing the CM section of the header.
///         Because GZIP only ever uses 8/DEFLATE, this is unneccessary,
///         but is present in the rare case where GZIP updates to contain
///         more compression methods.
/// * 'flg' - A bool array with 5 elements containing the 5 flags the FLG
///         section in the header. Takes the format [FTEXT, FHCRC, FEXTRA
///         FNAME, FCOMMENT].
/// * 'mtime' A u32 containing the modification timestamp in unix format.
///         Might be 0 if not defined in the file.
/// * 'xfl' - A single u8 representing whether the compression algorithim
///         used was the most compressing or fastest, either 2 or 4.
/// * 'os' - A single u8 value defining the operating system. Also mostly
///         useless nowadays.
/// * 'crc' - An optional u16 containing the CRC16 checksum if provided.
/// * 'fextra' - An optional Vec<u8> containing the extra flags if provided.
/// * 'fname' - An optional String containing the name of the original file.
/// * 'fcomment' - An optional String containing the files comment if provided.
pub struct GzipHeader {
    pub cm: u8,
    pub flg: [bool; 5],
    pub mtime: u32,
    pub xfl: u8,
    pub os: u8,
    pub crc: Option<u16>,
    pub fextra: Option<Vec<u8>>,
    pub fname: Option<String>,
    pub fcomment: Option<String>,
}

impl GzipHeader {
    pub fn build(bytes: &[u8]) -> Result<Self, GzipError> {
        // Initally define header as the first 10 bytes, then later append any
        // optional header elements.
        let mut header = bytes[0..10].to_vec();

        // Split the header into each part.
        let id = [header[0], header[1]];
        let cm = header[2];
        let flg = header[3];
        let mtime = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let xfl = header[8];
        let os = header[9];

        // Check for the ID bytes.
        if id != [0x1f, 0x8b] {
            return Err(GzipError::NotGzipFile([header[0], header[1]].to_vec()));
        }

        // Optional values.
        let mut _crc: Option<u16> = None;
        let mut _fextra: Option<Vec<u8>> = None;
        let mut _fname: Option<String> = None;
        let mut _fcomment: Option<String> = None;

        // Bool array for the possible flags.
        // [0] = FTEXT
        // [1] = FHCRC
        // [2] = FEXTRA
        // [3] = FNAME
        // [4] = FCOMMENT
        let mut flags: [bool; 5] = [false; 5];
        let mut idx = 10;
        match flg {
            ftext if ftext & 0b0000_0001 == 0b0000_0001 => {
                flags[0] = true;
            }
            fextra if fextra & 0b0000_0100 == 0b0000_0100 => {
                let xlen: usize = u16::from_le_bytes([bytes[10], bytes[11]]) as usize;
                _fextra = Some(bytes[12..xlen].to_vec());
                flags[2] = true;
            }
            fname if fname & 0b0000_1000 == 0b0000_1000 => {
                idx = match &_fextra {
                    Some(v) => idx + v.len(),
                    None => idx,
                };

                let after_header = bytes[idx..]
                    .iter()
                    .map(|x| x.to_owned())
                    .take_while(|byte| byte != &0u8)
                    .collect::<Vec<_>>();

                _fname = match String::from_utf8(after_header) {
                    Ok(str) => Some(str),
                    Err(_) => {
                        return Err(GzipError::InvalidHeader(header));
                    }
                };
                flags[3] = true;
            }
            fcomment if fcomment & 0b0001_0000 == 0b0001_0000 => {
                idx = match &_fname {
                    Some(v) => idx + v.len(),
                    None => idx,
                };

                let after_header = bytes[idx..]
                    .iter()
                    .map(|x| x.to_owned())
                    .take_while(|byte| byte != &0u8)
                    .collect::<Vec<_>>();

                _fcomment = match String::from_utf8(after_header) {
                    Ok(str) => Some(str),
                    Err(_) => {
                        return Err(GzipError::InvalidHeader(header));
                    }
                };

                flags[4] = true;
            }
            fhcrc if fhcrc & 0b0000_0010 == 0b0000_0010 => {
                idx = match &_fcomment {
                    Some(v) => idx + v.len(),
                    None => idx,
                };

                _crc = Some(((bytes[idx] as u16) << 8) | bytes[idx + 1] as u16);

                flags[1] = true;
            }
            _ => {}
        };
        Ok(Self {
            cm,
            flg: flags,
            mtime,
            xfl,
            os,
            crc: _crc,
            fextra: _fextra,
            fname: _fname,
            fcomment: _fcomment,
        })
    }
}
/// A struct containing the parts of a gzip file.
///
/// # Fields
///
/// * 'header' - A byte vector containing the header, has to be a vector due to
///         gzip optional header elements.
/// * 'deflate' - A byte vector containing the DEFLATE compressed blocks.
/// * 'footer' - A byte vector containing the footer
pub struct GzipFile {
    pub header: GzipHeader,
    pub deflate: Vec<u8>,
}

impl GzipFile {
    pub fn new(bytes: &[u8]) -> Result<Self, GzipError> {
        let header = GzipHeader::build(bytes)?;
        Ok(Self {
            header,
            deflate: vec![0],
        })
    }
}

#[derive(Debug)]
pub enum GzipError {
    InvalidHeader(Vec<u8>),
    NotGzipFile(Vec<u8>),
}

// Define how GzipErrors are displayed.
impl Display for GzipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GzipError::InvalidHeader(header) => {
                write!(f, "Error: Failed to parse header bytes {:?}", header)
            }
            GzipError::NotGzipFile(magic_bytes) => {
                write!(
                    f,
                    "Error: File missing GZIP ID bytes '{}, {}' does not equal '0x1f, 0x8b",
                    magic_bytes[0], magic_bytes[1]
                )
            }
        }
    }
}

impl Error for GzipError {}
