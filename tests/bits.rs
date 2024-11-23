use gzip::bitstream::BitStream;

//#[test]
#[allow(dead_code)]
fn test_bitstream() {
    let mut stream = BitStream::new();

    let bits: [u8; 8] = [1, 0, 1, 0, 1, 0, 1, 0];

    for bit in bits {
        stream.push_bit(bit);
    }

    let partial_byte: u8 = 0b0000_1111;
    stream.push_partial(partial_byte, 4, 7).unwrap();
    stream.push_partial(partial_byte, 1, 4).unwrap();

    let bytes = stream.bytes;

    let concatenated: u16 = ((bytes[0] as u16) << 8) | bytes[1] as u16;
    assert_eq!(concatenated, 0b10101010_11110001);
}
