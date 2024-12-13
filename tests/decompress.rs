use std::fs;

use gzip::gzip::GzipFile;

#[test]
fn test_block_type_0() {
    let mut compressed = GzipFile::from_path("./tests/compressed/block_type_0.gz").unwrap();
    let raw = fs::read("./tests/raw/block_type_0").unwrap();

    let decompressed = compressed.decompress().unwrap();

    assert_eq!(raw, decompressed);
}

#[test]
fn test_block_type_1() {
    let mut compressed_1 = GzipFile::from_path("./tests/compressed/block_type_1.gz").unwrap();
    let mut compressed_2 = GzipFile::from_path("./tests/compressed/block_type_1_lzss.gz").unwrap();

    let raw_1 = fs::read("./tests/raw/block_type_1").unwrap();
    let raw_2 = fs::read("./tests/raw/block_type_1_lzss").unwrap();

    let decompressed_1 = compressed_1.decompress().unwrap();
    let decompressed_2 = compressed_2.decompress().unwrap();

    assert_eq!(raw_1, decompressed_1);
    assert_eq!(raw_2, decompressed_2);
}

#[test]
fn test_block_type_2() {
    let mut compressed_1 = GzipFile::from_path("./tests/compressed/block_type_2.gz").unwrap();
    let mut compressed_2 = GzipFile::from_path("./tests/compressed/block_type_2_long.gz").unwrap();

    let raw_1 = fs::read("./tests/raw/block_type_2").unwrap();
    let raw_2 = fs::read("./tests/raw/block_type_2_long").unwrap();

    let decompressed_1 = compressed_1.decompress().unwrap();
    let decompressed_2 = compressed_2.decompress().unwrap();

    assert_eq!(raw_1, decompressed_1);
    assert_eq!(raw_2, decompressed_2);
}

#[test]
fn test_larger_file() {
    let mut compressed = GzipFile::from_path("./tests/compressed/picture.png.gz").unwrap();

    let raw = fs::read("./tests/raw/picture.png").unwrap();

    let decompressed = compressed.decompress().unwrap();

    assert_eq!(raw, decompressed);
}
