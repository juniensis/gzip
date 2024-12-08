use criterion::{criterion_group, criterion_main, Criterion};

use std::fs;

use gzip::gzip::GzipFile;

fn test_block_type_0() {
    let mut compressed = GzipFile::from_path("./tests/compressed/block_type_0.gz").unwrap();
    let raw = fs::read("./tests/raw/block_type_0").unwrap();

    let decompressed = compressed.decompress().unwrap();

    assert_eq!(raw, decompressed);
}

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

#[allow(clippy::redundant_closure)]
fn benchmark(c: &mut Criterion) {
    c.bench_function("Type 0", |b| b.iter(|| test_block_type_0()));
    c.bench_function("Type 1", |b| b.iter(|| test_block_type_1()));
    c.bench_function("Type 2", |b| b.iter(|| test_block_type_2()));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
