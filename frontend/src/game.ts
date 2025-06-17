import { BOARD_SIZE, DEFAULT_FEN, TILE_SIZE } from './constants';
import { Event, EventListener } from './event-manager';
import * as BitboardX from '../../pkg/bitboard_x';
import { Point2D } from './utils';
import { RuntimeModule, runtime } from './runtime';

export type SelectedPiece = {
  piece: string;
  x: number;
  y: number;
  file: number; // from file
  rank: number; // from rank
};

export class Game implements EventListener, RuntimeModule {
  private game: BitboardX.WasmGameState | null;
  private _selected: SelectedPiece | null;
  private canvas: HTMLCanvasElement | null;
  private _board: string;

  public constructor() {
    this.reset();
  }

  public get board() {
    return this._board;
  }

  public set board(value) {
    if (value === this._board) {
      return;
    }
    this._board = value;

    // eslint-disable-next-line no-console
    console.log(this.game!.debug_string());
    const undoButton = document.getElementById('undoButton') as HTMLButtonElement;
    if (undoButton) {
      undoButton.disabled = !this.game!.can_undo();
    }
    const redoButton = document.getElementById('redoButton') as HTMLButtonElement;
    if (redoButton) {
      redoButton.disabled = !this.game!.can_redo();
    }
  }

  public getName(): string {
    return 'Game';
  }

  public get selectedPiece() {
    return this._selected;
  }

  public init(): boolean {
    this.reset();
    this.game = new BitboardX.WasmGameState();
    runtime.eventManager.addListener(this);
    return this.restart();
  }

  public tick() {
    if (this.game?.game_over()) {
      // eslint-disable-next-line no-console
      console.log('Game over!');
      return;
    }

    this.game?.tick();
    this.board = this.game!.to_board_string();
  }

  private reset() {
    this.game = null;
    this._selected = null;
    this.canvas = null;
    this._board = '';
  }

  public restart(): boolean {
    const fen = (document.getElementById('fenInput') as HTMLInputElement).value || DEFAULT_FEN;
    try {
      this.game!.reset_game(fen, false, false);
      this.board = this.game!.to_board_string();
      this.canvas = runtime.display.canvas;
      return true;
    } catch (e) {
      // eslint-disable-next-line no-console
      console.error(`Error parsing FEN '${fen}': ${e}`);
      return false;
    }
  }

  private onMouseDown(event: Point2D) {
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

    this.canvas!.style.cursor = 'grabbing';

    this._selected = { piece, ...event, rank, file };
  }

  private onMouseMove(event: Point2D) {
    const { x, y } = event;

    if (this._selected !== null) {
      this._selected.x = x;
      this._selected.y = y;
    }
  }

  private onMouseUp(event: Point2D) {
    this.canvas!.style.cursor = 'grab';

    const selected = this._selected;
    this._selected = null;

    const x = Math.floor(event.x / TILE_SIZE);
    const y = Math.floor(event.y / TILE_SIZE);

    if (x < 0 || x >= BOARD_SIZE || y < 0 || y >= BOARD_SIZE) {
      return;
    }
    if (selected === null) {
      return;
    }

    const { file, rank } = selected;
    const file2 = x;
    const rank2 = 7 - y;

    let promotion = '';
    if (rank === 6 && rank2 === 7 && selected.piece.toLowerCase() === 'p') {
      promotion = prompt("Enter what piece to promote: ") || '';
    }

    const move = `${String.fromCharCode(97 + file)}${rank + 1}${String.fromCharCode(97 + file2)}${rank2 + 1}${promotion}`;
    this.game!.inject_move(move);
  }

  private undo() {
    if (this.game!.undo()) {
      this.board = this.game!.to_board_string();
    }
  }

  private redo() {
    if (this.game!.redo()) {
      this.board = this.game!.to_board_string();
    }
  }

  public handleEvent(event: Event) : void {
    switch (event.type) {
    case 'mousedown':
      this.onMouseDown(event.payload as Point2D);
      break;
    case 'mousemove':
      this.onMouseMove(event.payload as Point2D);
      break;
    case 'mouseup':
      this.onMouseUp(event.payload as Point2D);
      break;
    case 'undo':
      this.undo();
      break;
    case 'redo':
      this.redo();
      break;
    case 'restart':
      this.restart();
      break;
    default:
      // eslint-disable-next-line no-console
      console.warn(`Unhandled event type: ${event.type}`);
      break;
    }
  }
}
