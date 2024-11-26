use std::fs;

use ::gzip::{deflate, gzip::GzipFile};

#[test]
fn test_header() {
    let file = fs::read("./tests/data/block_type_0.gz").unwrap();
    let file_1 = fs::read("./tests/data/block_type_1_noname.gz").unwrap();

    let gzfile = GzipFile::build(&file).unwrap();
    let gzfile_1 = GzipFile::build(&file_1).unwrap();
    let header = gzfile.header;
    let header_1 = gzfile_1.header;

    let attributes = format!(
        "{} {:?} {} {} {} {:?} {:?} {:?} {:?} {}",
        header.cm,
        header.flg,
        header.mtime,
        header.xfl,
        header.os,
        header.crc,
        header.fextra,
        header.fname,
        header.fcomment,
        header.end_idx
    );

    let attributes_1 = format!(
        "{} {:?} {} {} {} {:?} {:?} {:?} {:?} {}",
        header_1.cm,
        header_1.flg,
        header_1.mtime,
        header_1.xfl,
        header_1.os,
        header_1.crc,
        header_1.fextra,
        header_1.fname,
        header_1.fcomment,
        header_1.end_idx
    );

    println!("{}", attributes_1);

    assert_eq!(attributes, "8 [false, false, false, true, false] 1732227461 0 255 None None Some(\"block_type_0\") None 23");
}

#[test]
fn test_block_type_0() {
    let type_0 = fs::read("./tests/data/block_type_0.gz").unwrap();

    let file_0 = GzipFile::build(&type_0).unwrap();

    let block_0 = deflate::DeflateBlock::build(&file_0.deflate).unwrap();

    let decompressed_0 = block_0.decompress().unwrap();

    let decompressed_0_string = String::from_utf8_lossy(&decompressed_0);

    assert_eq!(decompressed_0_string, "Lorem ipsum");
}
#[test]
fn test_block_type_1() {
    // Test case "Lorem ipsum".
    let bytes_1 = fs::read("./tests/data/block_type_1.gz").unwrap();
    let file_1 = GzipFile::build(&bytes_1).unwrap();
    let block_1 = deflate::DeflateBlock::build(&file_1.deflate).unwrap();
    let decompressed_1 = block_1.decompress().unwrap();
    let decompressed_1_string = String::from_utf8_lossy(&decompressed_1);

    // Test case "AABBBBCCCCCCCC"
    let bytes_2 = fs::read("./tests/data/block_type_1_noname.gz").unwrap();
    let file_2 = GzipFile::build(&bytes_2).unwrap();
    let block_2 = deflate::DeflateBlock::build(&file_2.deflate).unwrap();
    let decompressed_2 = block_2.decompress().unwrap();
    let decompressed_2_string = String::from_utf8_lossy(&decompressed_2);

    assert_eq!(decompressed_1_string, "Lorem ipsum");
    assert_eq!(decompressed_2_string, "AABBBBCCCCCCCC\n");
}
#[ignore = "reason"]
#[test]
fn test_block_type_2() {
    let type_2 = fs::read("./tests/data/block_type_2.gz").unwrap();

    let file_2 = GzipFile::build(&type_2).unwrap();

    let block_2 = deflate::DeflateBlock::build(&file_2.deflate).unwrap();

    let decompressed_2 = block_2.decompress().unwrap();

    let decompressed_2_string = String::from_utf8_lossy(&decompressed_2);

    assert_eq!(
        decompressed_2_string,
        "Lorem impsum dolor sit amet, consectetur adipiscing elit."
    );
}
