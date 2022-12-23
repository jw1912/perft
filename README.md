# perft
This repository contains a selection of perft implemtations for varying choice of board representation.
The aim of the code is to remain as simple as possible whilst not sacrificing too much performance.
All the implementations are self-contained and require no dependencies.

#### Compiling
Run ```cargo build --release```, if you have cargo installed, to compile all binaries.

#### What is perft?
Perft is a simple test to see if move generation and making/unmaking moves works correctly.
For any position, perft to a given depth counts the number of leaf nodes in the game tree, achieved by making strictly legal moves to that depth.
