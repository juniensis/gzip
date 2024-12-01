//! Gzip encoding and decoding.
use std::{error::Error, fmt::Display, fs, path::Path};

use crate::inflate::DeflateData;

/// A custom error type for GZIP related errors.
///
/// # Members
///
/// * 'InvalidHeader' - Used when the header bytes are read, but something
///             goes wrong while parsing them. Contains a Vec<u8> holding
///             the bytes of the invalid header.
/// * 'NotGzipFile' - Used when a file is read that does not contain the GZIP
///             magic bytes (0x1f, 0x8b).
/// * 'IoError' - Wrapper for std::io::Error.
#[derive(Debug)]
pub enum GzipError {
    InvalidHeader(Vec<u8>),
    NotGzipFile(Vec<u8>),
    IoError(std::io::Error),
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
            GzipError::IoError(err) => {
                write!(f, "Error: Operation raised the io::Error: {}", err)
            }
        }
    }
}

impl Error for GzipError {}

impl From<std::io::Error> for GzipError {
    fn from(err: std::io::Error) -> Self {
        GzipError::IoError(err)
    }
}

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
#[derive(Debug)]
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
    pub end_idx: usize,
}

impl GzipHeader {
    /// Accepts the raw bytes from a GZIP file and parses out the header
    /// elements.
    ///
    /// # Arguments
    ///
    /// * 'bytes' - A reference to the byte array containing the header.
    ///
    /// # Returns
    ///
    /// Either the successfully built header, or a GzipError due to either
    /// failing to parse the header, or the bytes lacking the GZIP file
    /// identification bytes.
    pub fn build(bytes: &[u8]) -> Result<Self, GzipError> {
        // Extract the core 10 byte header.
        let header = bytes[0..10].to_vec();

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

        // Initialize the option values as none.
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
        flags
            .iter_mut()
            .enumerate()
            .for_each(|(x, y)| *y = matches!((flg >> x) & 1, 1));

        // The byte index to update as optional elements are found.
        let mut _idx: usize = 10;

        // If FEXTRA is set, collect the two bytes that dictate its size,
        // and then take that amount of bytes from the data stream.
        if flags[2] {
            let xlen = u16::from_le_bytes([bytes[10], bytes[11]]);
            _fextra = Some(bytes[12..xlen as usize].to_vec());
            _idx += xlen as usize + 2;
        }

        if flags[3] {
            let after_header = bytes[_idx..]
                .iter()
                .cloned()
                .take_while(|byte| byte != &0u8)
                .collect::<Vec<_>>();

            _idx += after_header.len() + 1;

            _fname = match String::from_utf8(after_header) {
                Ok(name) => Some(name),
                Err(_) => {
                    return Err(GzipError::InvalidHeader(header));
                }
            }
        }

        if flags[4] {
            let after_header = bytes[_idx..]
                .iter()
                .cloned()
                .take_while(|byte| byte != &0u8)
                .collect::<Vec<_>>();

            _idx += after_header.len() + 1;

            _fcomment = match String::from_utf8(after_header) {
                Ok(comment) => Some(comment),
                Err(_) => {
                    return Err(GzipError::InvalidHeader(header));
                }
            }
        }

        // Now check for FHCRC because it occurs at the end of the header
        // right before the DEFLATE data, so, _idx needs to be incremented as
        // much as it will be before grabbing the crc.
        if flags[1] {
            _crc = Some(((bytes[_idx] as u16) << 8) | bytes[_idx + 1] as u16);

            _idx += 2;
        }

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
            end_idx: _idx,
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
    pub deflate: DeflateData,
    pub crc32: u32,
    pub isize: u32,
}

impl GzipFile {
    /// Accepts a byte array and returns a GzipFile struct.
    ///
    /// # Arguments
    ///
    /// * 'bytes' - A reference to a byte array containing the gzip file.
    ///
    /// # Returns
    ///
    /// The built GzipFile struct, or an error if building the header failed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, GzipError> {
        let header = GzipHeader::build(bytes)?;
        let footer = &bytes[bytes.len() - 8..bytes.len()];

        let crc32 = u32::from_le_bytes([footer[0], footer[1], footer[2], footer[3]]);
        let isize = u32::from_le_bytes([footer[4], footer[5], footer[6], footer[7]]);
        println!("{}", header.end_idx);
        let deflate_raw = bytes[header.end_idx..bytes.len() - 8].to_vec();

        Ok(Self {
            header,
            deflate: DeflateData::build(&deflate_raw),
            crc32,
            isize,
        })
    }
    /// Accepts a path, extracts the bytes, and returns the built file from
    /// those bytes.
    ///
    /// # Arguments
    ///
    /// * 'path' - A path in the form of any type that can be coerced into a
    ///         Path.
    ///
    /// # Returns
    ///
    /// Either the GzipFile struct, or a GzipError.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<GzipFile, GzipError> {
        let bytes = fs::read(path)?;

        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::GzipFile;
    #[ignore]
    #[test]
    fn test_gzip_file() {
        let file = GzipFile::from_path("./tests/data/block_type_0.gz").unwrap();
    }
}
