# Bitboards

#### Board-Representation
The board is stored as 8 64-bit unsigned integers, which repspectively represent the occupied squares for
white pieces, black pieces and each of the 6 piece types, where a bit in the number being one represents that
the board is occupied by that piece (/ a piece of that colour, for the colour bitboards). The pieces of each colour
can be found by simply intersecting the relevant piece and colour bitboards. The ordering used here is H8 is the
most significant bit (MSB), then decreases across the rank before going to the next one, with the least significant
bit at A1.

```
Pawn bitboard:
         H8       H1      A8       A1
         |        |       |        |
(MSB) 0b|1|111111|1| ... |1|111111|1|

White bitboard:
(MSB) 0b00000000 ... 11111111 -> LSB

White pawns bitboard = Pawns bitboard & White bitboard:
(MSB) 0b00000000 ... 11111111 -> LSB
```
