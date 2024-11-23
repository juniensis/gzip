use gzip::bitstream::{self, BitStream};

#[test]
fn test_bitstream() {
    let mut stream = BitStream::new();

    let bits: [u8; 8] = [1, 0, 1, 0, 1, 0, 1, 0];

    for bit in bits {
        stream.push_bit(bit);
    }

    let partial_byte: u8 = 0b0000_1111;
    stream.push_partial(partial_byte, 4, 7).unwrap();
    stream.push_partial(partial_byte, 1, 4).unwrap();

    println!("{}", stream);

    for x in stream {
        println!("{:08b}", x);
    }
}
