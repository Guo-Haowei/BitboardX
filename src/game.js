import { BOARD_SIZE, CANVAS_ID, DEFAULT_FEN, TILE_SIZE } from './constants.js';
import { Engine } from '../engine/pkg/engine.js';
import { renderer } from './renderer.js';
import { printBoard } from './utility.js';

export class Game {
  constructor() {
    this.engine = new Engine();
    this.selected = null;
    this.boardString = '';
    this.canvas = document.getElementById(CANVAS_ID);
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

  onMouseDown(event) {
    const x = Math.floor(event.x / TILE_SIZE);
    const y = Math.floor(event.y / TILE_SIZE);
    if (x < 0 || x >= BOARD_SIZE || y < 0 || y >= BOARD_SIZE) {
      return;
    }

    const index = x + y * BOARD_SIZE;

    const piece = this.boardString[index];
    if (piece === '.') {
      return;
    }

    const rank = 7 - y;
    const file = x;
    this.selected = file + rank * BOARD_SIZE;
    this.canvas.style.cursor = 'grabbing';

    const moves = this.engine.gen_moves(this.selected);
    renderer.setSelectedPiece({ piece, x: event.x, y: event.y, moves });
  }

  onMouseMove(event) {
    const { x, y } = event;
    if (renderer.selectedPiece) {
      renderer.selectedPiece.x = x;
      renderer.selectedPiece.y = y;
    }
  }

  onMouseUp(event) {
    renderer.unsetSelectedPiece();
    this.canvas.style.cursor = 'grab';

    const x = Math.floor(event.x / TILE_SIZE);
    const y = Math.floor(event.y / TILE_SIZE);

    if (x < 0 || x >= BOARD_SIZE || y < 0 || y >= BOARD_SIZE) {
      return;
    }
    if (this.selected === null) {
      return;
    }

    let file = this.selected % BOARD_SIZE;
    let rank = Math.floor(this.selected / BOARD_SIZE);
    const from = `${String.fromCharCode(97 + file)}${rank + 1}`;
    rank = 7 - y;
    file = x;
    const to = `${String.fromCharCode(97 + file)}${rank + 1}`;
    const square = file + rank * BOARD_SIZE;

    console.log(`${from}${to}`);
    if (this.engine.apply_move(this.selected, square)) {
      // Update the board status here
      this.boardString = this.engine.to_string();
      printBoard(this.boardString);
    }

    this.selected = null;
  }
}
