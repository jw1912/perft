# perft
This repository contains a selection of perft implementations for varying choice of board representation.
The aim of the code is to remain as simple as possible whilst not sacrificing too much performance.

All the implementations are self-contained and require no dependencies; see the individual READMEs for
how each implementation works.

### What is perft?
Perft is a simple test to see if move generation and making/unmaking moves works correctly.
For any position, perft to a given depth counts the number of leaf nodes in the game tree, achieved by making strictly legal moves to that depth.

## Compiling
Run ```cargo build --release```, if you have cargo installed, to compile all binaries.

**Note:** ```quad-bitboard``` requires rust nightly, as it uses the portable_simd feature.

## Board Representation

### General Layout
Fullmove and halfmove counters are not included in the board representation, as they are not needed for perft.
```rust
pub struct Position {
    board: ...,    // the 8x8 board, this is what is different between representations
    c: bool,       // side to move
    enp: u8,       // en passant square, 0 if none
    cr: u8,        // castling rights
}
```

### Piece-Centric vs Square-Centric
#### Piece-Centric
- [```bitboards```](/bitboard)
- piece-lists
- piece-sets
#### Square-Centric
- mailbox
    + 0x88
    + 8x8
    + general padded (e.g. [```10x12```](/mailbox-10x12))
#### Mix
- [```quad-bitboards```](/quad-bitboard)



## Consistency

### State
En-passant squares, castling rights, and the side to move are handled identically in all implementations.

### Move Generation
All move generation is pseudo-legal, and moves are checked for legality after they are made, being undone if they are illegal.
As a result no "tricks" are used to inflate NPS.

### Copy/make vs Make/unmake
Copy/make is used for simplicity (and it is usually faster for perft).
