# Gzip Encoder/Decoder

## Goals
### Decode
1. [ ] Parse header/footer.
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
