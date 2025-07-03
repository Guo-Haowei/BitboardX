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

## TODO

- [ ] Implement **Null Move Pruning** to speed up the search by pruning obvious losing moves
- [ ] Add **Late Move Reductions (LMR)** to reduce search depth for less promising moves
- [ ] Improve the **Evaluation Function** with more nuanced heuristics
- [ ] Integrate **NNUE (Neural Network Unified Evaluator)** for advanced evaluation and better playing strength
