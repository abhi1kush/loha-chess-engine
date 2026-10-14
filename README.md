# loha

Rust port of the C chess engine in `my_first_chessengine`.

The board is the same 120-square mailbox. Move generation, make and unmake, piece values, piece-square tables, and alpha-beta search follow that engine. It speaks UCI, XBoard, and a small console.

```
cargo build --release
./target/release/loha
```

Then type `uci`, `xboard`, or `cons`.

```
uci
position startpos
go depth 6
```

Perft from the starting position, depth 5, is 4,865,609. The bundled suite passes through depth 4:

```
cargo run --release -- perft 5
cargo run --release -- perft 4 "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"
cargo run --release -- suite 4
```

If `performance.bin` (a Polyglot book) is in the working directory, the engine plays book moves. Pass `NoBook` on the command line, or set the UCI `Book` option, to turn that off.

In the console, moves are entered as `e2e4` or `e7e8q`. `help` lists the other commands.
