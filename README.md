# GZIP Decoder

## Goals

### Decode

- [x] Parse header.

- [x] DEFLATE
  - [x] Read block header.
  - [x] Process block type 0.
  - [x] Process block type 1.
    - [x] Generate fixed prefix code tree.
    - [x] Bitstream to symbols.
    - [x] Decode LZSS.
  - [x] Process block type 2.
    - [x] Generate code length prefix code tree.
    - [x] Generate literal/length prefix code tree.
    - [x] Generate distance prefix code tree.
    - [x] Bitstream to symbols.
    - [x] Decode LZSS

- [x] Confirm CRC-32 checksum

### Problems to Fix

- [ ] Really slow.
  - [ ] Remove as many instances of cloning as possible primarily within the
          walk and insert_code functions.
- [ ] Seems to break on not tiny files.
  - [x] Checked if it's because the LZSS lookup buffer spans all blocks.
    - [x] Switched decoding loop to looking back through all decoded
              data rather than just the current block.
    - [x] Still doesn't work.

## Benchmarks

| Block Type | Test Size | Time | Megabytes per Second |
|------------|-----------|------|----------------------|
| 0          | 47 Bytes  | 6.98 µs | 6.7335 Mb/S       |
| 1          | 124 Bytes | 66.32 µs | 1.870 Mb/S       |
| 2          | 457 Bytes | 46.85 ms | 9.7541 Mb/S       |

## 1. The GZIP Format

### 1.1 Introduction

The GZIP format is a lossless file compression data format used within the GZIP utility
described in RFC 1952. GZIP files are made up by a series of members each made up of
a header, a set of DEFLATE compressed blocks, with the end marked
by the DEFLATE blocks footer.

### 1.2 The Header

The header has a core 10 byte set of data, followed by optional data. The format
is described in RFC 1952 section 2.3 as follows:

         +---+---+---+---+---+---+---+---+---+---+
         |ID1|ID2|CM |FLG|     MTIME     |XFL|OS | (more-->)
         +---+---+---+---+---+---+---+---+---+---+

      (if FLG.FEXTRA set)

         +---+---+=================================+
         | XLEN  |...XLEN bytes of "extra field"...| (more-->)
         +---+---+=================================+

      (if FLG.FNAME set)

         +=========================================+
         |...original file name, zero-terminated...| (more-->)
         +=========================================+

      (if FLG.FCOMMENT set)

         +===================================+
         |...file comment, zero-terminated...| (more-->)
         +===================================+

      (if FLG.FHCRC set)

         +---+---+
         | CRC16 |
         +---+---+

ID1 and ID2 are the bytes that mark the following data as a GZIP compressed file.
They fixed as the bytes 0x1f and 0x8b. CM is the compression method, and is a single
byte, values 0-7 are reserved, but the only value used is 8, or the byte 0x08. Then
there is FLG which is broken down into individual bits with each bit meaning the
following:

      bit 0   FTEXT
      bit 1   FHCRC
      bit 2   FEXTRA
      bit 3   FNAME
      bit 4   FCOMMENT
      bit 5   reserved
      bit 6   reserved
      bit 7   reserved

FTEXT is optional and signifies that the file is ASCII text, the rest are described
in the description of the header format above. MTIME is 4 bytes and is the modification
time in UNIX format (seconds since 00:00:00 GMT, Jan. 1, 1970.) and is also optional.
XFL represents whether the compression used is the most compressed, or fastest algorithm,
represented by the bytes 0x02 and 0x04 respectively. Finally, OS is a single byte representing
which operating system is used, values 0-13 are reserved and 255 is reserved for
unknown. The majority of these operating systems no longer exist. However the full list is on page
8 of the RFC.

### 1.3 The Body

The body of a GZIP file is made up of a number of DEFLATE compressed blocks, which will be explained
in their own section.

### 1.4 The Trailer

At the end of every GZIP member is a CRC32 check value, and a section describing the number of bytes in
the original uncompressed data. This is referred to as the trailer, and takes the following format:

           0   1   2   3   4   5   6   7
         +---+---+---+---+---+---+---+---+
         |     CRC32     |     ISIZE     |
         +---+---+---+---+---+---+---+---+

CRC32 is the check value of the original uncompressed data computed from the CRC-32 algorithm. It's
implementation in this program will be described if/when it is implemented, however, in the mean time, the Wikipedia page
on cyclic redundancy checks (<https://en.wikipedia.org/wiki/Cyclic_redundancy_check>) provides a good
example of encoding a 14 bit message with a 3 bit CRC with the polynomial x^3 + x + 1:
  
    Start with the message to be encoded: 
    
    11010011101100

    This is first padded with zeros corresponding to the bit length n of the CRC. This is done so that 
    the resulting code word is in systematic form. Here is the first calculation for computing a 3-bit CRC: 

    11010011101100 000 <--- input right padded by 3 bits
    1011               <--- divisor (4 bits) = x³ + x + 1
    ------------------
    01100011101100 000 <--- result

    The algorithm acts on the bits directly above the divisor in each step. The result for that iteration 
    is the bitwise XOR of the polynomial divisor with the bits above it. The bits not above the divisor are 
    simply copied directly below for that step. The divisor is then shifted right to align with the highest 
    remaining 1 bit in the input, and the process is repeated until the divisor reaches the right-hand end 
    of the input row. Here is the entire calculation: 

    11010011101100 000 <--- input right padded by 3 bits
    1011               <--- divisor
    01100011101100 000 <--- result (note the first four bits are the XOR with the divisor beneath, the rest of the bits are unchanged)
    1011              <--- divisor ...
    00111011101100 000
      1011
    00010111101100 000
      1011
    00000001101100 000 <--- note that the divisor moves over to align with the next 1 in the dividend (since quotient for that step was zero)
          1011             (in other words, it doesn't necessarily move one bit per iteration)
    00000000110100 000
            1011
    00000000011000 000
            1011
    00000000001110 000
              1011
    00000000000101 000
              101 1
    -----------------
    00000000000000 100 <--- remainder (3 bits).  Division algorithm stops here as dividend is equal to zero.

    Since the leftmost divisor bit zeroed every input bit it touched, when this process ends the only bits in 
    the input row that can be nonzero are the n bits at the right-hand end of the row. These n bits are the
    remainder of the division step, and will also be the value of the CRC function (unless the chosen CRC 
    specification calls for some postprocessing).

    The validity of a received message can easily be verified by performing the above calculation again, 
    this time with the check value added instead of zeroes. The remainder should equal zero if there 
    are no detectable errors. 

    11010011101100 100 <--- input with check value
    1011               <--- divisor
    01100011101100 100 <--- result
    1011              <--- divisor ...
    00111011101100 100

    ......

    00000000001110 100
              1011
    00000000000101 100
              101 1
    ------------------
    00000000000000 000 <--- remainder

ISIZE is a good bit simpler, and is just a 4 byte little endian value representing the length of the original uncompressed
data. So for a length of 11 the last 4 bytes of a GZIP file would look like:

    0b0000_1011 0b0000_0000 0b0000_0000 0b0000_0000

## 2. Bits into Bytes

Now for a brief detour. In both .gz data and the DEFLATE bitstream, bits are packed in to bytes as follows:

    1. The first bit is pushed as the least significant bit of the byte.
    2. Successive bits are then pushed in increasing significance, so the
      eighth bit becomes the most significant bit of the byte.
    3. Once the byte is full, the byte is output and the process repeats.

So when looking at the hexdump of a GZIP file, remember that the actual bitstream looks like if every byte
was flipped from little-endian to big-endian. For example, the bitstream for the example ISIZE given above
would look like:
  
    1101_0000_0000_0000_0000_0000

You might notice that 1101 equals 13, not 11, and this is because of rule #1.

### 2.1 Rule #1

A devious yet consistent little scheme exists within both GZIP and DEFLATE: No matter what, any numerical
value is pushed into the bitstream least significant bit to most significant bit. This does not apply to
arbitrary bit sequences such as prefix codes or symbols, but does apply to values such as ISIZE, the CRC-32
check value, and once we get to DEFLATE, things like distance/length offsets, the LEN in block type 0, and more.
This is something that is somewhat easy to forget but will quickly ruin an implementation.

## 3. DEFLATE

### 3.1 Introduction

DEFLATE is a lossless file compression data format described originally by the memo RFC 1951. The core concepts
behind DEFLATE, is LZSS encoding, and  prefix codes. The concepts are somewhat incorrectly called "LZ77" and
"Huffman codes" in RFC 1951. The algorithm referred to as LZ77 better matches the Lempel–Ziv–Storer–Szymanski algorithm
which is a derivative of LZ77 that performs checks to ensure the token generated is more space efficient than just
outputting the literal value. Inversely likewise, what are referred to as Huffman Codes, are derived from bit lengths rather than
a frequency based Huffman tree, making "prefix codes" the more correct terminology as utilized in the papers
RFC 1951 references. Beyond semantics, this is just useful to avoid my mistake of implementing frequency based
Huffman trees, before finding out they won't be particularly useful for decoding. Anyways, the DEFLATE data format
is made up of an arbitrary number of blocks of various types containing a header, a compressed data stream (with bits
pushed into it as described in section 2), and an EOB marker except for block type 0. The header format changes
depending on the block type yet all headers begin with three bits defining whether the block is the last, and
what type the block is.

### 3.2 Block Types

The 3 bit header contains two variables: BFINAL and BTYPE. BFINAL is represented by the first bit, and BTYPE
is represented by the following two. So, all block types will at least have the following elements in common:

          BFINAL BTYPE BITSTREAM...
    bits: 1      2

There are 3 block types, note that the block types are considered a numerical value and therefore follow
rule #1.

    00 - No compression.
    01 - Compressed with fixed prefix codes.
    10 - Compressed with dynamic prefix codes.
    11 - Reserved (error).

Because of rule #1 when looking at the bytes for a compressed block the bits will appear as is, however,
they will be flipped when looking at the bitstream.

### 3.2.1 Block Type 0

Data in block type 0 is uncompressed and byte-aligned, so after the 3 bit header there will be 5 bits of
padding, followed by 2 values LEN and NLEN both of which take up two bytes. LEN is the total length of
uncompressed data present and NLEN is the bitwise complement to LEN. The presence of NLEN at best is only
useful for double checking that the block is valid, otherwise it can realistically be ignored and maintain
compliance. After NLEN the bitstream begins, and is not terminated by an EOB marker, rather LEN is used to
decide how much data should be collected. So, the pseudocode to read block type 0 is as follows (presuming
the block has already been confirmed to be of type 0):

    output = []
    // Conversion from bytes to u16, the least significant byte comes first.
    // [0b0000_0001, 0b_0000_0000] -> 0b0000_0000_0000_0000 | 0b0000_0000_0000_0001 -> 1u16
    len = (input[2] as u16 << 8) | input[1] as u16
    nlen = (input[4] as u16 << 8) | input[3] as u16
    // Optional check that len == ~nlen.
    if len == ~nlen:
      for byte in input[5:len]:
        output.append(byte)

    return output

Here is an example of decoding a DEFLATE block of type 0:

    input:
    bytes: 0x01 0x03 0x00 0xFC 0xFF 0x41 0x42 0x43
    bitstream: 1000_0000_1100_0000_0000_0000_0011
    1111_1111_1111 0100_0001 0100_0010 0100_0011

    bitstream elements:
      Here we can see that the block is of type 0, therefore there will be
      a section for LEN and NLEN after 5 bits of padding.
      BFINAL/BTYPE/PADDING: 1000_0000
      Because LEN is a number it is pushed in the bitstream LSB first,
      so with how the conversion from bytes to bitstream occurs, the value
      is held LSB on the right in the bytes in this case 0x00 0x03 = 3.
      LEN: 1100_0000_0000_0000
      NLEN: 0011_1111_1111_1111
      We know the next 3 bytes are the complete data in the file.
      DATA: 0100_0001 0100_0010 0100_0011

    output:
    0x41 0x42 0x43

The final bytes in the data are the ASCII characters "ABC".

### 3.2.2 Block Type 1

Block type 1 is the first to implement the core ideas of DEFLATE. First an EOB marker (256) is added
to the end of the input data. Then the input data is tokenized by the LZSS algorithm into symbols
representing lengths and distances specified by a length and distance code table given in section
3.2.5 of RFC 1951. Next these symbols are encoded using a fixed prefix code table from section 3.2.6
of RFC 1951. The decompression process is the inverse of this process, section 3.2.3 of the RFC,
provides the following pseudocode:

      loop (until end of block code recognized)
          decode literal/length value from input stream
          if value < 256
            copy value (literal byte) to output stream
          otherwise
            if value = end of block (256)
                break from loop
            otherwise (value = 257..285)
                decode distance from input stream

                move backwards distance bytes in the output
                stream, and copy length bytes from this
                position to the output stream.
      end loop

For an example, lets decompress some data by hand.

    input:
      bytes: 0x73 0x74 0x74 0x02 0x02 0x67 0x28 0xe0 0x02 0x00
      bitstream: 1100_1110_0010_1110_0010_1110_0100_0000_0100_0000_1110_0110
      0001_0100_0000_0111_0100000000_ 000000 <- After EOB can be truncated.

      bitstream elements:
        Taking the first three bits and flipping them before reading right to left.
        BFINAL/BTYPE: 110 -> 011 -> BFINAL: 1 BTYPE: 01
        DATA: 0111000101110001011100100000001000000111001100001010000000111010
        EOB: 0000000 (Prefix code for 256)

To decompress this block we'll need the prefix code table:

    Lit Value    Bits        Codes
    ---------    ----        -----
      0 - 143     8          00110000 through
                            10111111
    144 - 255     9          110010000 through
                            111111111
    256 - 279     7          0000000 through
                            0010111
    280 - 287     8          11000000 through
                            11000111

The length code table:

         Extra               Extra               Extra
    Code Bits Length(s) Code Bits Lengths   Code Bits Length(s)
    ---- ---- ------     ---- ---- -------   ---- ---- -------
      257   0     3       267   1   15,16     277   4   67-82
      258   0     4       268   1   17,18     278   4   83-98
      259   0     5       269   2   19-22     279   4   99-114
      260   0     6       270   2   23-26     280   4  115-130
      261   0     7       271   2   27-30     281   5  131-162
      262   0     8       272   2   31-34     282   5  163-194
      263   0     9       273   3   35-42     283   5  195-226
      264   0    10       274   3   43-50     284   5  227-257
      265   1  11,12      275   3   51-58     285   0    258
      266   1  13,14      276   3   59-66

Finally, the distance code table:

         Extra           Extra               Extra
    Code Bits Dist  Code Bits   Dist     Code Bits Distance
    ---- ---- ----  ---- ----  ------    ---- ---- --------
      0   0    1     10   4     33-48    20    9   1025-1536
      1   0    2     11   4     49-64    21    9   1537-2048
      2   0    3     12   5     65-96    22   10   2049-3072
      3   0    4     13   5     97-128   23   10   3073-4096
      4   1   5,6    14   6    129-192   24   11   4097-6144
      5   1   7,8    15   6    193-256   25   11   6145-8192
      6   2   9-12   16   7    257-384   26   12  8193-12288
      7   2  13-16   17   7    385-512   27   12 12289-16384
      8   3  17-24   18   8    513-768   28   13 16385-24576
      9   3  25-32   19   8   769-1024   29   13 24577-32768

Reference table for all of the fixed codes is at the end of this file (Fig 1.) The process is as follows:

  1.Read bitstream starting after the header from left to right.
  2.Follow down the tree until a symbol is reached.
    - For block type 1, using a tree is somewhat unnecessary, rather you can filter the table until you have only one match.
  3. Output the symbols to a temporary output stream.
  4. Perform LZSS decoding using the length and distance symbols.

    DATA: 01110001 // Code for 65 / A
          01110001
          01110010 // Code for 66 / B
          0000001 // Code for 257 / length of 3
          00000 // Distance code 0
          01110011 // Code for 67 / C
          0000101 // Code for 261 / length 7
          00000 // Distance code 0
          00111010 // Code for 10 / Null terminator

    SYMBOLS: 65 65 66 257 0 67 261 0 10

    A A B <3, 1> C <7, 1> 0x0a -> AABBBBCCCCCCCC0x0a

The final output seems to be a text file with the text "AABBBBCCCCCCCC" with a null terminator.
This case only uses low length and distance codes, so the extra bits on each aren't used. After
a length code, there might be a number of extra bits given in the table, after reading the length
symbol, then read the amount of extra bits given, and the number is the offset. So, to represent
the length 12, the symbol 265 is used and has 1 extra bit. If that bit is 0 it will be 11, if it
is 1 it will be 12. So after encoding with the fixed prefix code table the bitstream would look
like: 00010011. After a length symbol there is always a distance, which is represented by a symbol
from 0-29 with extra bits defined by the table. So to represent a length of 12 and a distance of 8
the bitstream would look like: 00010011001101001 with 0001001 1 being the length code with the extra
bit, and 00110100 1 being the distance code with the extra bit.

### 3.2.3 Block Type 2

Block type 2 behaves quite similarly to block type 1, however, with the enormous difference of
encoding the prefix code lengths within the header. The process of encoding block type 2 consists
of taking the input data, adding an EOB marker, and applying LZSS to generate literals and length/
distance pairs. Then, based on the data generate prefix codes for the lengths/literals and the
distances, before taking the code lengths of these prefix codes, and creating prefix codes for the
lengths of the prefix codes (confusing right?). The code lengths are then encoded into the header,
and finally the input data can be encoded into the bitstream using the literal/length and distance
prefix codes. This all is very confusing, so let's just start with outlining what a block of type
2 looks like.

          BFINAL  BTYPE  HLIT  HDIST  HCLEN    CL Lengths    LL Lengths     Dist Lengths
    BITS: 1       2      5     5      4       (HCLEN + 4)*3  HLIT + 257     HDIST + 1
    
          DATA ... EOB

So, after the 3 bit header all blocks share, there is HLIT, which represents how many code lengths
are present for literals and lengths. Every block needs to specify either a code or the absence of a
code for every byte, and a code for EOB, so HLIT contains the number of total literals/lengths - 257.
HDIST contains the number of distance codes - 1. HCLEN describes the number of code length codes - 4,
the code lengths are encoded as 3 bit integers directly after HCLEN, in the field CL Lengths. Then,
with the code lengths in CL Lengths, a prefix code tree is built and used to parse the LL and Dist
lengths. Lets decode a block by hand, this will take some time.

    1010001010110011100000111001000011000011000011000011000010100000000010
    1111011110111001010001111111000000001000001111001010001011110001101001
    0111011100000111001000110100010000111110000010011011010011000110100100
    1010011001111111011001010010111100001011100000000111100011110111101001    
    1011010010010111000001110010111100110111000101100110010101101101000001
    1100110000001010110111010010001101100001110000110110000111010011100010
    0001001000100111010000101011011000111001001011111100000111000111001000
    0100010100110001010000011110111001010101011010110111000111010100101011
    1001100110001010110000100000110000010111010100010101100111100110010100
    1101100000111010010110000001101111100101001101010110001010010000000101
    0011101110100110100110000110010110110101000100000110001110111010000011
    0110101000001111001100100101111111101110000101100011011101001100010001
    1101111011111000011011000111111000111000111010001100010101010011111110
    11111010000011110001111100

    BFINAL: 1
    BTYPE: 01 -> 10 = 2
    HLIT: 00010 -> 01000 = 8 
    HDIST: 10110 -> 01101 = 13
    HCLEN: 0111 -> 1110 = 14

    Then take (14 + 4) * 3 bits to get the CL Lengths.

    000  001  110  010  000  110  000  110  000  110  000  110  000  101  000  000  000  101

    Flip due to rule #1.

    000  100  011  010  000  011  000  011  000  011  000  011  000  101  000  000  000  101
    0    4    3    2    0    3    0    3    0    3    0    3    0    5    0    0    0    5

Now we get to one of the most devious parts of block type 2. Without specifying why, the RFC
just states that the order these code lengths appear in is as follows: 16, 17, 18, 0, 8, 7, 9,
6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15.

    16:  17:  18:  0:   8:   7:   9:   6:   10:  5:   11:  4:   12:  3:   13:  2:   14:  1:
    000  100  011  010  000  011  000  011  000  011  000  011  000  101  000  000  000  101
    0    4    3    2    0    3    0    3    0    3    0    3    0    5    0    0    0    5

    Sort them back into a normal order.
    0...                                                   18
    [2, 5, 0, 5, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 3]

    Now it's time to generate the codes and build the prefix code tree.

    First get the number of occurances of each code length, ignoring 0.
    
    0: 0, 1: 0, 2: 1, 3: 5, 4: 1, 5: 2
  
    Now generate the next codes for each code length. This is done by iterating over
    each nonzero code length, from lowest to highest, and setting its next code as
    the previous code + the number the last code length appears, and bitshifting over
    1.

    code = 0
    1: 0
      The next code with a length of 1 is 0 + the number of times a bit length of 0
      appears (0) << 1.
    code = 0
    2: 00
      0 + the number of times a code length of 1 appears (0) << 1.
    code = 0
    3: 010 = 2
      (0 + 1) << 1
    4: 1110 = 7
      (2 + 5) << 1
    5: 11110 = 8
      (7 + 1) << 1

    [0, 0, 00, 010, 1110, 11110]

    Now we can generate the codes for each code length. To do this, iterate through the sorted
    code length array from above, and if the length does not equal 0, assign the next code for
    that code length and iterate the next code for the code length.
    
    [2, 5, 0, 5, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 3]

    [00, 11110, 0, 11111, 010, 011, 100, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1110, 110]

    Now we can insert these codes into a binary tree with their indexes being their values
    and we'll get the following tree.

    ┌── (1)
    │   ├── (11)
    │   │   ├── (111)
    │   │   │   ├── (1111)
    │   │   │   │   ├── (11111: 3)
    │   │   │   │   └── (11110: 1)
    │   │   │   └── (1110: 17)
    │   │   └── (110: 18)
    │   └── (10)
    │       ├── (101: 7)
    │       └── (100: 6)
    └── (0)
        ├── (01)
        │   ├── (011: 5)
        │   └── (010: 4)
        └── (00: 0)

    These values come from the following alphabet:

    0 - 15: Represent code lengths of 0 - 15
        16: Copy the previous code length 3 - 6 times.
            The next 2 bits indicate repeat length
                  (0 = 3, ... , 3 = 6)
              Example:  Codes 8, 16 (+2 bits 11),
                        16 (+2 bits 10) will expand to
                        12 code lengths of 8 (1 + 6 + 5)
        17: Repeat a code length of 0 for 3 - 10 times.
            (3 bits of length)
        18: Repeat a code length of 0 for 11 - 138 times
            (7 bits of length)

    The following process would be comically time consuming, so I'll only decode the first 
    few LL code lengths.

    1110 -> 17
    111 -> 7
    The next 3 + 7 codes (indices 0..=9) have a length of 0. 
    
    101 -> 7
    The code length for literal value 10 (ASCII line feed) is 7.
    
    110 -> 18
    0101000 -> 0001010 = 10
    The next 10 + 11 codes (indices 11..=31) have a length of 0.

    11111 -> 3
    The code length for literal value 32 (ASCII space) is 3.
    
This process continues until HLIT + 257 and HDIST + 1 code lengths have been decoded.
An important thing to note is that the repeat sequences caused by symbols 16..=18 can continue
past the boundary between the LL code lengths and the Dist code lengths, so it's best to decode
the full block, and then split the output to get the separate LL and Dist code length arrays.

After decoding the code lengths for the literal/length and distance codes, the prefix code trees
can be built, and the main bitstream can be decoded. The process for decoding the bitstream is
nearly identical to decoding block type 1, however, because there is a prefix code tree for the
distance codes, you must decode the distance code following a length symbol, rather than just taking
the next 5 bits.

## Extra

Fig 1. Unabridged fixed prefix code table.

    +---------------------------------------------------------------------------+
    |  0000000  | 256  |  0000001  | 257  |  0000010  | 258  |  0000011  | 259  |
    |  0000100  | 260  |  0000101  | 261  |  0000110  | 262  |  0000111  | 263  |
    |  0001000  | 264  |  0001001  | 265  |  0001010  | 266  |  0001011  | 267  |
    |  0001100  | 268  |  0001101  | 269  |  0001110  | 270  |  0001111  | 271  |
    |  0010000  | 272  |  0010001  | 273  |  0010010  | 274  |  0010011  | 275  |
    |  0010100  | 276  |  0010101  | 277  |  0010110  | 278  |  0010111  | 279  |
    | 00110000  |  0   | 00110001  |  1   | 00110010  |  2   | 00110011  |  3   |
    | 00110100  |  4   | 00110101  |  5   | 00110110  |  6   | 00110111  |  7   |
    | 00111000  |  8   | 00111001  |  9   | 00111010  |  10  | 00111011  |  11  |
    | 00111100  |  12  | 00111101  |  13  | 00111110  |  14  | 00111111  |  15  |
    | 01000000  |  16  | 01000001  |  17  | 01000010  |  18  | 01000011  |  19  |
    | 01000100  |  20  | 01000101  |  21  | 01000110  |  22  | 01000111  |  23  |
    | 01001000  |  24  | 01001001  |  25  | 01001010  |  26  | 01001011  |  27  |
    | 01001100  |  28  | 01001101  |  29  | 01001110  |  30  | 01001111  |  31  |
    | 01010000  |  32  | 01010001  |  33  | 01010010  |  34  | 01010011  |  35  |
    | 01010100  |  36  | 01010101  |  37  | 01010110  |  38  | 01010111  |  39  |
    | 01011000  |  40  | 01011001  |  41  | 01011010  |  42  | 01011011  |  43  |
    | 01011100  |  44  | 01011101  |  45  | 01011110  |  46  | 01011111  |  47  |
    | 01100000  |  48  | 01100001  |  49  | 01100010  |  50  | 01100011  |  51  |
    | 01100100  |  52  | 01100101  |  53  | 01100110  |  54  | 01100111  |  55  |
    | 01101000  |  56  | 01101001  |  57  | 01101010  |  58  | 01101011  |  59  |
    | 01101100  |  60  | 01101101  |  61  | 01101110  |  62  | 01101111  |  63  |
    | 01110000  |  64  | 01110001  |  65  | 01110010  |  66  | 01110011  |  67  |
    | 01110100  |  68  | 01110101  |  69  | 01110110  |  70  | 01110111  |  71  |
    | 01111000  |  72  | 01111001  |  73  | 01111010  |  74  | 01111011  |  75  |
    | 01111100  |  76  | 01111101  |  77  | 01111110  |  78  | 01111111  |  79  |
    | 10000000  |  80  | 10000001  |  81  | 10000010  |  82  | 10000011  |  83  |
    | 10000100  |  84  | 10000101  |  85  | 10000110  |  86  | 10000111  |  87  |
    | 10001000  |  88  | 10001001  |  89  | 10001010  |  90  | 10001011  |  91  |
    | 10001100  |  92  | 10001101  |  93  | 10001110  |  94  | 10001111  |  95  |
    | 10010000  |  96  | 10010001  |  97  | 10010010  |  98  | 10010011  |  99  |
    | 10010100  | 100  | 10010101  | 101  | 10010110  | 102  | 10010111  | 103  |
    | 10011000  | 104  | 10011001  | 105  | 10011010  | 106  | 10011011  | 107  |
    | 10011100  | 108  | 10011101  | 109  | 10011110  | 110  | 10011111  | 111  |
    | 10100000  | 112  | 10100001  | 113  | 10100010  | 114  | 10100011  | 115  |
    | 10100100  | 116  | 10100101  | 117  | 10100110  | 118  | 10100111  | 119  |
    | 10101000  | 120  | 10101001  | 121  | 10101010  | 122  | 10101011  | 123  |
    | 10101100  | 124  | 10101101  | 125  | 10101110  | 126  | 10101111  | 127  |
    | 10110000  | 128  | 10110001  | 129  | 10110010  | 130  | 10110011  | 131  |
    | 10110100  | 132  | 10110101  | 133  | 10110110  | 134  | 10110111  | 135  |
    | 10111000  | 136  | 10111001  | 137  | 10111010  | 138  | 10111011  | 139  |
    | 10111100  | 140  | 10111101  | 141  | 10111110  | 142  | 10111111  | 143  |
    | 11000000  | 280  | 11000001  | 281  | 11000010  | 282  | 11000011  | 283  |
    | 11000100  | 284  | 11000101  | 285  | 11000110  | 286  | 11000111  | 287  |
    | 110010000 | 144  | 110010001 | 145  | 110010010 | 146  | 110010011 | 147  |
    | 110010100 | 148  | 110010101 | 149  | 110010110 | 150  | 110010111 | 151  |
    | 110011000 | 152  | 110011001 | 153  | 110011010 | 154  | 110011011 | 155  |
    | 110011100 | 156  | 110011101 | 157  | 110011110 | 158  | 110011111 | 159  |
    | 110100000 | 160  | 110100001 | 161  | 110100010 | 162  | 110100011 | 163  |
    | 110100100 | 164  | 110100101 | 165  | 110100110 | 166  | 110100111 | 167  |
    | 110101000 | 168  | 110101001 | 169  | 110101010 | 170  | 110101011 | 171  |
    | 110101100 | 172  | 110101101 | 173  | 110101110 | 174  | 110101111 | 175  |
    | 110110000 | 176  | 110110001 | 177  | 110110010 | 178  | 110110011 | 179  |
    | 110110100 | 180  | 110110101 | 181  | 110110110 | 182  | 110110111 | 183  |
    | 110111000 | 184  | 110111001 | 185  | 110111010 | 186  | 110111011 | 187  |
    | 110111100 | 188  | 110111101 | 189  | 110111110 | 190  | 110111111 | 191  |
    | 111000000 | 192  | 111000001 | 193  | 111000010 | 194  | 111000011 | 195  |
    | 111000100 | 196  | 111000101 | 197  | 111000110 | 198  | 111000111 | 199  |
    | 111001000 | 200  | 111001001 | 201  | 111001010 | 202  | 111001011 | 203  |
    | 111001100 | 204  | 111001101 | 205  | 111001110 | 206  | 111001111 | 207  |
    | 111010000 | 208  | 111010001 | 209  | 111010010 | 210  | 111010011 | 211  |
    | 111010100 | 212  | 111010101 | 213  | 111010110 | 214  | 111010111 | 215  |
    | 111011000 | 216  | 111011001 | 217  | 111011010 | 218  | 111011011 | 219  |
    | 111011100 | 220  | 111011101 | 221  | 111011110 | 222  | 111011111 | 223  |
    | 111100000 | 224  | 111100001 | 225  | 111100010 | 226  | 111100011 | 227  |
    | 111100100 | 228  | 111100101 | 229  | 111100110 | 230  | 111100111 | 231  |
    | 111101000 | 232  | 111101001 | 233  | 111101010 | 234  | 111101011 | 235  |
    | 111101100 | 236  | 111101101 | 237  | 111101110 | 238  | 111101111 | 239  |
    | 111110000 | 240  | 111110001 | 241  | 111110010 | 242  | 111110011 | 243  |
    | 111110100 | 244  | 111110101 | 245  | 111110110 | 246  | 111110111 | 247  |
    | 111111000 | 248  | 111111001 | 249  | 111111010 | 250  | 111111011 | 251  |
    | 111111100 | 252  | 111111101 | 253  | 111111110 | 254  | 111111111 | 255  |
    +---------------------------------------------------------------------------+
