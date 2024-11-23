use std::fs;

use ::gzip::{deflate, gzip::GzipFile};
use gzip::gzip;

#[test]
fn test_header() {
    let file = fs::read("./tests/data/block_type_0.gz").unwrap();

    let gzfile = GzipFile::build(&file).unwrap();

    /*
    println!(
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

    for byte in gzfile.deflate {
        println!("{:08b}", byte);
    }
    */
}

#[test]
fn test_deflate() {
    let type_0 = fs::read("./tests/data/block_type_0.gz").unwrap();
    let type_1 = fs::read("./tests/data/block_type_1.gz").unwrap();
    let type_2 = fs::read("./tests/data/block_type_2.gz").unwrap();

    let file_0 = GzipFile::build(&type_0).unwrap();
    let file_1 = GzipFile::build(&type_1).unwrap();
    let file_2 = GzipFile::build(&type_2).unwrap();

    let block_0 = deflate::DeflateBlock::build(&file_0.deflate).unwrap();
    let block_1 = deflate::DeflateBlock::build(&file_1.deflate).unwrap();
    let block_2 = deflate::DeflateBlock::build(&file_2.deflate).unwrap();

    let decompressed_0 = block_0.decompress().unwrap();
}
