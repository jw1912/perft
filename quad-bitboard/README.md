# perft
A fast chess perft program, written in Rust.

#### Quad-Bitboards
This is the quad bitboard branch, where instead of as 8 bitboards the board is stored as 4 bitboards in a single simd vector (u64x4).
Pieces are stored vertically, with one bit in each of the four boards, each piece type having its own nibble encoding.

In its current state this implementation is around 5% slower than the equivalent standard bitboard approach, but only with ```target-cpu=native``` on a modern processor - compiling both implementations without targeting avx2, etc sees it fall further behind. Unfortunately, quad bitboards are also not particularly versatile, so likely aren't suitable for an actual engine (where copy-make becomes an issue anyway due to the massive ramp up in memory traffic from hash tables and the like).

#### Compiling
Run ```cargo build --release``` if you have cargo installed.

#### What is perft?
For any position, perft to a given depth counts the number of leaf nodes in the game tree, achieved by making strictly legal moves.

#### Features
- No unsafe code
- Quad-bitboards
- Pseudo-legal move generation
- Copy-make
- Hyperbola quintessence sliding piece attacks
