# perft
This repository contains a selection of perft implementations for varying choice of board representation.
The aim of the code is to remain as simple as possible whilst not sacrificing too much performance.

All the implementations are self-contained and require no dependencies; see the individual READMEs for
how each implementation works.

#### Compiling
Run ```cargo build --release```, if you have cargo installed, to compile all binaries.

#### What is perft?
Perft is a simple test to see if move generation and making/unmaking moves works correctly.
For any position, perft to a given depth counts the number of leaf nodes in the game tree, achieved by making strictly legal moves to that depth.

## Consistency

#### Move Generation
All move generation is pseudo-legal, and moves are checked for legality after they are made, being undone if they are illegal.
As a result no "tricks" are used to inflate NPS.

In the case of bitboards vs quad-bitboards, the move generation is identical.

#### Copy/make vs Make/unmake
Whichever is faster will be used, with copy/make preferred due to its simplicity if the margin is close. In an actual engine make/unmake is usually
better because of the added memory pressure of everything else in search, but in perft copy/make is often faster.
