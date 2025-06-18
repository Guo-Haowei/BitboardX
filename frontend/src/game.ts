import { BOARD_SIZE, DEFAULT_FEN, TILE_SIZE } from './constants';
import { Event, EventListener } from './event-manager';
import * as BitboardX from '../../pkg/bitboard_x';
import { Point2D, fileRankToString } from './utils';
import { RuntimeModule, runtime } from './runtime';

export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number; // from file
  rank: number; // from rank
  legalMoves: Set<string>;
}

export class Game implements EventListener, RuntimeModule {
  private game: BitboardX.WasmGame | null;
  private _selected: SelectedPiece | null;
  private canvas: HTMLCanvasElement | null;
  private _board: string;
  private moveLookup = new Map<string, Set<string>>();

  public constructor() {
    this.game = null;
    this._selected = null;
    this.canvas = null;
    this._board = '';
  }

  public get board() {
    return this._board;
  }

  public set board(value) {
    if (!this.game) {
      console.error('Game not initialized. Cannot set board.');
      return;
    }

    if (value === this._board) {
      return;
    }
    this._board = value;
    this.moveLookup.clear();

    const legalMoves = this.game.get_legal_moves();
    legalMoves.forEach((move) => {
      const from = move.slice(0, 2);
      const to = move.slice(2, 4);
      if (!this.moveLookup.has(from)) {
        this.moveLookup.set(from, new Set());
      }
      this.moveLookup.get(from)?.add(to);
    });

    console.log(this.game.debug_string());
    const undoButton = document.getElementById('undoButton') as HTMLButtonElement;
    if (undoButton) {
      undoButton.disabled = !this.game.can_undo();
    }
    const redoButton = document.getElementById('redoButton') as HTMLButtonElement;
    if (redoButton) {
      redoButton.disabled = !this.game.can_redo();
    }
  }

  public getName(): string {
    return 'Game';
  }

  public get selectedPiece() {
    return this._selected;
  }

  public init(): boolean {
    runtime.eventManager.addListener(this);
    this.game = new BitboardX.WasmGame();
    return this.restart();
  }

  public tick() {
    if (!this.game || !this.board) {
      return;
    }
    this.game.tick();
    this.board = this.game.to_board_string();

    // @TODO: better game result handling
    if (this.game?.game_over()) {
      alert('Game over!');
      this.restart();
      return;
    }
  }

  public restart(): boolean {
    if (!this.game) {
      console.error('Game not initialized. Cannot restart.');
      return false;
    }

    const fen = (document.getElementById('fenInput') as HTMLInputElement).value || DEFAULT_FEN;
    try {
      const isPlayerHuman = (player: string) => {
        const element = document.querySelector(`input[name="${player}"]:checked`);
        if (element && element instanceof HTMLInputElement) {
          return element.value === 'human';
        }
        return false;
      };

      this.game.reset_game(fen, isPlayerHuman('player1'), isPlayerHuman('player2'));
      this.board = this.game.to_board_string();
      this.canvas = runtime.display.canvas;
      return true;
    } catch (e) {

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

    if (this.canvas) {
      this.canvas.style.cursor = 'grabbing';
    }

    const legalMoves = this.moveLookup.get(fileRankToString(file, rank)) || new Set();

    this._selected = { piece, ...event, rank, file, legalMoves };
  }

  private onMouseMove(event: Point2D) {
    const { x, y } = event;

    if (this._selected !== null) {
      this._selected.x = x;
      this._selected.y = y;
    }
  }

  private onMouseUp(event: Point2D) {
    if (this.canvas) {
      this.canvas.style.cursor = 'grab';
    }

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

    const move = `${fileRankToString(file, rank)}${fileRankToString(file2, rank2)}${promotion}`;
    this.game?.inject_move(move);
  }

  private undo() {
    if (this.game?.undo()) {
      this.board = this.game.to_board_string();
    }
  }

  private redo() {
    if (this.game?.redo()) {
      this.board = this.game.to_board_string();
    }
  }

  public handleEvent(event: Event): void {
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

        console.warn(`Unhandled event type: ${event.type}`);
        break;
    }
  }
}
