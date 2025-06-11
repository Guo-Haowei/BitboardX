import { BOARD_SIZE, CANVAS_ID, DEFAULT_FEN, TILE_SIZE } from './constants.js';
import * as Engine from '../engine/pkg/engine.js';
import { renderer } from './renderer.js';

// @TODO: come up with a better name
export class Game {
  constructor() {
    this.engine = null;
    this.selected = null;
    this.canvas = document.getElementById(CANVAS_ID);

    this._board = '';
  }

  get board() {
    return this._board;
  }

  set board(value) {
    this._board = value;
    console.log(this.engine.to_string(false));
    document.getElementById('undoButton').disabled = !this.engine.can_undo();
    document.getElementById('redoButton').disabled = !this.engine.can_redo();
  }

  init(fen) {
    fen = fen || DEFAULT_FEN;
    console.log(`Initializing game with FEN: ${fen}`);
    try {
      this.engine = new Engine.Game(fen);
      this.board = this.engine.to_board_string();
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

    const piece = this.board[index];
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
    let rank2 = 7 - y;
    let file2 = x;

    const move =  `${String.fromCharCode(97 + file)}${rank + 1}${String.fromCharCode(97 + file2)}${rank2 + 1}`;
    console.log(move);
    if (this.engine.execute(move)) {
      this.board = this.engine.to_board_string();
    }

    this.selected = null;
  }

  undo() {
    if (this.engine.undo()) {
      this.board = this.engine.to_board_string();
    }
  }

  redo() {
    if (this.engine.redo()) {
      this.board = this.engine.to_board_string();
    }
  }
}
