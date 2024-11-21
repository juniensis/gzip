# Gzip Encoder/Decoder

## Goals
### Decode
- [ ] Parse header/footer.
- [ ] DEFLATE
  1. [ ] Read block header.
  2. [ ] Process block type 0.
  3. [ ] Process block type 1.
      1. [ ] Generate fixed prefix codes.
      2. [ ] Bytes to bitstream.
      3. [ ] Bitstream to symbols.
      4. [ ] LZSS
  - [ ] Process block type 2.
      1. [ ] Generate dynamic prefix codes.
      2. [ ] Bytes to bitstream.
      3. [ ] Bitstream to symbols.
      4. [ ] LZSS
- [ ] Output.
