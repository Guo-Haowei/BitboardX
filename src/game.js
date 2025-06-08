import { BOARD_SIZE, DEFAULT_FEN } from './constants.js';
import { Engine } from '../engine/pkg/engine.js';
import { renderer } from './renderer.js'

export class Game {
  constructor() {
    this.engine = new Engine();
    this.selectedSquare = -1;
    this.boardString = '';
  }

  init(fen) {
    fen = fen || DEFAULT_FEN;
    console.log(`Initializing game with FEN: ${fen}`);
    try {
      this.engine.parse_fen(fen);
      this.boardString = this.engine.to_string();
      return true;
    } catch (e) {
      console.error(`Error parsing FEN '${fen}': ${e}`);
      return false;
    }
  }

  onClick(row, col) {
    const boardString = this.engine.to_string();
    const square = col + row * BOARD_SIZE;

    // If selected square is the same as clicked square, do nothing
    if (square === this.selectedSquare) {
      return;
    }

    const selectedPiece = boardString[square];
    this.selectedSquare = square;

    const rank = 8 - row;
    const file = col;

    console.log(`Selected piece: ${selectedPiece} at ${String.fromCharCode(file + 97)}${rank}`);

    const moves = this.engine.gen_moves(col, row);
    console.log(`Moves type: ${typeof(moves)}`);
    console.log(`Generated moves: ${moves}`);

    renderer.draw({
      boardString: this.boardString,
      selectedSqaure: square,
      moves,
    });
  }
}
