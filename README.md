# perft
A simple, but fast, chess perft program, written in Rust.

#### Compiling
Run ```cargo build --release``` if you have cargo installed.

#### What is perft?
For any position, perft to a given depth counts the number of leaf nodes in the game tree, achieved by making strictly legal moves.

#### Features
- No unsafe code
- Bitboard-based (6 piece bitboards + 2 colour bitboards)
- Pseudo-legal move generation
- Copy-make
- Hyperbola quintessence sliding piece attacks
