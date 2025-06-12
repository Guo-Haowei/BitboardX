import { BOARD_SIZE, DEFAULT_FEN, TILE_SIZE } from './constants';
import { Event, EventListener } from './event-manager';
import * as BitboardX from '../../pkg/bitboard_x';
import { Point2D } from './utils';
import { RuntimeModule, runtime } from './runtime';

export class Game implements EventListener, RuntimeModule {
  private game: BitboardX.Game | null;
  private selected: number;
  private canvas: HTMLCanvasElement | null;
  private _board: string;

  public constructor() {
    this.reset();
  }

  public get board() {
    return this._board;
  }

  public set board(value) {
    this._board = value;
    // eslint-disable-next-line no-console
    console.log(this.game!.to_string(false));
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

  public init(): boolean {
    this.reset();
    runtime.eventManager.addListener(this);
    return this.restart(DEFAULT_FEN);
  }

  public tick() {
  }

  private reset() {
    this.game = null;
    this.game = null;
    this.selected = -1;
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
    this.selected = file + rank * BOARD_SIZE;
    this.canvas!.style.cursor = 'grabbing';

    const moves = this.game!.gen_moves(this.selected);
    runtime.renderer.setSelectedPiece({ piece, ...event, moves });
  }

  private onMouseMove(event: Point2D) {
    const { x, y } = event;
    const { renderer } = runtime;

    const selectedPiece = renderer.getSelectedPiece();
    if (selectedPiece) {
      const { piece, moves } = selectedPiece;
      renderer.setSelectedPiece({piece, x, y, moves});
    }
  }

  private onMouseUp(event: Point2D) {
    runtime.renderer.unsetSelectedPiece();
    this.canvas!.style.cursor = 'grab';

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
    if (this.game!.execute(move)) {
      this.board = this.game!.to_board_string();
    }

    this.selected = -1;
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
