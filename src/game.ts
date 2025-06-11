import { BOARD_SIZE, CANVAS_ID, TILE_SIZE } from './constants.js';
import * as Engine from '../engine/pkg/engine.js';
import { renderer } from './renderer.js';
import { Point2D } from './utils.js';

// @TODO: come up with a better name
export class Game {
  private engine: Engine.Game | null;
  private selected: number;
  private canvas: HTMLCanvasElement;
  private _board: string;

  constructor() {
    this.engine = null;
    this.selected = -1;
    this.canvas = document.getElementById(CANVAS_ID)! as HTMLCanvasElement;

    this._board = '';
  }

  public get board() {
    return this._board;
  }

  public set board(value) {
    this._board = value;
    // eslint-disable-next-line no-console
    console.log(this.engine!.to_string(false));
    const undoButton = document.getElementById('undoButton') as HTMLButtonElement;
    if (undoButton) {
      undoButton.disabled = !this.engine!.can_undo();
    }
    const redoButton = document.getElementById('redoButton') as HTMLButtonElement;
    if (redoButton) {
      redoButton.disabled = !this.engine!.can_redo();
    }
  }

  public init(fen: string): boolean {
    // eslint-disable-next-line no-console
    console.log(`Initializing game with FEN: ${fen}`);
    try {
      this.engine = new Engine.Game(fen);
      this.board = this.engine.to_board_string();
      return true;
    } catch (e) {
      // eslint-disable-next-line no-console
      console.error(`Error parsing FEN '${fen}': ${e}`);
      return false;
    }
  }

  public onMouseDown(event: Point2D) {
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

    const moves = this.engine!.gen_moves(this.selected);
    renderer.setSelectedPiece({ piece, ...event, moves });
  }

  public onMouseMove(event: Point2D) {
    const { x, y } = event;

    const selectedPiece = renderer.getSelectedPiece();
    if (selectedPiece) {
      const { piece, moves } = selectedPiece;
      renderer.setSelectedPiece({piece, x, y, moves});
    }
  }

  public onMouseUp(event: Point2D) {
    renderer.unsetSelectedPiece();
    this.canvas.style.cursor = 'grab';

    const x = Math.floor(event.x / TILE_SIZE);
    const y = Math.floor(event.y / TILE_SIZE);

    if (x < 0 || x >= BOARD_SIZE || y < 0 || y >= BOARD_SIZE) {
      return;
    }
    if (this.selected === -1) {
      return;
    }

    const file = this.selected % BOARD_SIZE;
    const rank = Math.floor(this.selected / BOARD_SIZE);
    const rank2 = 7 - y;
    const file2 = x;

    const move =  `${String.fromCharCode(97 + file)}${rank + 1}${String.fromCharCode(97 + file2)}${rank2 + 1}`;
    if (this.engine!.execute(move)) {
      this.board = this.engine!.to_board_string();
    }

    this.selected = -1;
  }

  public undo() {
    if (this.engine!.undo()) {
      this.board = this.engine!.to_board_string();
    }
  }

  public redo() {
    if (this.engine!.redo()) {
      this.board = this.engine!.to_board_string();
    }
  }
}
