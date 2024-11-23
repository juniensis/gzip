use gzip::prefix;

#[test]
fn prefix_generation() {
    // Create a bit_length array for the ASCII symbols ABCDEFGH,
    // with lengths 3, 3, 3, 3, 3, 2, 4, 4.
    let lengths = [3, 3, 3, 3, 3, 2, 4, 4];

    prefix::PrefixTree::from_lengths(&lengths);
}
