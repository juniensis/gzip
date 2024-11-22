use std::fs;

use ::gzip::gzip::GzipFile;
use gzip::gzip;

#[test]
fn test_header() {
    let file = fs::read("./tests/data/block_type_0.gz").unwrap();

    let gzfile = GzipFile::new(&file).unwrap();

    let header = gzfile.header;
    println!(
        "{} {:?} {} {} {} {:?} {:?} {:?} {:?}",
        header.cm,
        header.flg,
        header.mtime,
        header.xfl,
        header.os,
        header.crc,
        header.fextra,
        header.fname,
        header.fcomment
    );
}
