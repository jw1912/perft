# Quad-Bitboards

#### Board-Representation
The board is stored as 4 bitboards in a single simd vector (u64x4).
Pieces are stored vertically, with one bit in each of the four boards, each piece type having its own nibble encoding.

```
 H8                    A1
 |                     |
|1|1111111 ... 0000000|0| - 1 if piece is black
|1|0011001 ... 1000000|1| - 1 if rook, queen or king
|0|1110110 ... 0100001|0| - 1 if knight, bishop or king
|0|0101100 ... 0000000|0| - 1 if pawn, bishop or queen
```

Using some bitwise instructions it is possible to (relatively) quickly extract the standard 8-bitboard representation
of the board for use in move generation (and checking for check, to establish move legality).
