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
  legalMoves: number[];
};

export class Game implements EventListener, RuntimeModule {
  private game: BitboardX.Game | null;
  private _selected: SelectedPiece | null;
  private canvas: HTMLCanvasElement | null;
  private _board: string;
  private moveMap: Map<number, number[]> = new Map();

  public constructor() {
    this.reset();
  }

  public get board() {
    return this._board;
  }

  public set board(value) {
    this._board = value;

    // this.moveMap.clear();

    const moveMap = new Map<number, number[]>();
    const legalMoves = this.game!.legal_moves();
    legalMoves.forEach((move) => {
      const from = move.from_sq();
      const to = move.to_sq();
      if (!moveMap.has(from)) {
        moveMap.set(from, []);
      }
      moveMap.get(from)!.push(to);
    });

    this.moveMap = moveMap;

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
    runtime.eventManager.addListener(this);
    return this.restart(DEFAULT_FEN);
  }

  public tick() {
  }

  private reset() {
    this.game = null;
    this._selected = null;
    this.canvas = null;
    this._board = '';
  }

  public restart(fen: string): boolean {
    // eslint-disable-next-line no-console
    console.log(`Initializing game with FEN: ${fen}`);
    try {
      this.game = new BitboardX.Game(fen);
      this.board = this.game.to_board_string();
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
    const square = file + rank * BOARD_SIZE;

    this.canvas!.style.cursor = 'grabbing';

    this._selected = { piece, ...event, rank, file, legalMoves: this.moveMap.get(square) || [] };
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

    const { file, rank, legalMoves } = selected;

    const file2 = x;
    const rank2 = 7 - y;
    const to = file2 + rank2 * BOARD_SIZE;

    const move = `${String.fromCharCode(97 + file)}${rank + 1}${String.fromCharCode(97 + file2)}${rank2 + 1}`;

    legalMoves.forEach(dest => {
      if (dest === to) {
        this.game!.do_move(move);
        this.board = this.game!.to_board_string();
      }
    });
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
      this.restart(event.payload as string);
      break;
    default:
      // eslint-disable-next-line no-console
      console.warn(`Unhandled event type: ${event.type}`);
      break;
    }
  }
}
