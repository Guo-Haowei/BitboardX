# BitboardX

BitboardX is a chess engine written in rust. Current playing strength is around **Stockfish 1600 Elo**.

The project includes two main components:

- **Engine**: A UCI-compliant command-line tool.
- [**Web Demo**](https://guo-haowei.github.io/pages/chess/)

---

## Features

- Fast and efficient bitboard-based move generation and evaluation
- UCI (Universal Chess Interface) support for easy integration with popular chess GUIs
- Web demo for instant play and testing without any setup

---

## Backend

### Build and Run

```bash
$ cargo run   # run UCI
$ cargo build # build
$ cargo test  # test
```

## Frontend

### Build and Run

```bash
$ wasm-pack build --target web # build wasm
$ cd frontend/
$ npm install
$ npm run dev
$ npm run build
```
