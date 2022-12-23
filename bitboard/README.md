# Bitboards

#### Board-Representation
The board is stored as 8 64-bit unsigned integers, which repspectively represent the occupied squares for
white pieces, black pieces and each of the 6 piece types, where a bit in the number being one represents that
the board is occupied by that piece (/ a piece of that colour, for the colour bitboards). The pieces of each colour
can be found by simply intersecting the relevant piece and colour bitboards.

```
Pawn bitboard:
        H8                    A1
        |                     |
MSB <- |1|1111111 ... 1111111|1| -> LSB

White bitboard:
        H8                    A1
        |                     |
MSB <- |0|0000000 ... 1111111|1| -> LSB

White pawns bitboard = Pawns bitboard & White bitboard:
        H8                    A1
        |                     |
MSB <- |0|0000000 ... 1111111|1| -> LSB
```
