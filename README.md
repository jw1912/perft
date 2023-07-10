# perft

A fast perft implementation.

This perft is single-threaded, optional bulk counting with legal movegen.

## What is perft?
Perft is a simple test to see if move generation and making/unmaking moves works correctly.
For any position, perft to a given depth counts the number of leaf nodes in the game tree, achieved by making strictly legal moves to that depth.

## Compiling
Run ```cargo build --release```, if you have cargo installed, to compile the binary.
