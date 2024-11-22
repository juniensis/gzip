# GZIP Encoder/Decoder

## Goals
### Decode
1. [x] Parse header.
2. [ ] DEFLATE
  - [ ] Read block header.
  - [ ] Process block type 0.
  - [ ] Process block type 1.
      1. [ ] Generate fixed prefix codes.
      2. [ ] Bytes to bitstream.
      3. [ ] Bitstream to symbols.
      4. [ ] LZSS
  - [ ] Process block type 2.
      1. [ ] Generate dynamic prefix codes.
      2. [ ] Bytes to bitstream.
      3. [ ] Bitstream to symbols.
      4. [ ] LZSS
3. [ ] Output.

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
