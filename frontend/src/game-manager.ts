import { DEFAULT_FEN } from './constants';
import { RuntimeModule, runtime } from './runtime';
import { Listener, EVENT_MAP } from './message-queue';
import { Board } from './board';
import { WasmGame } from '../../bitboard_x/pkg/bitboard_x';

export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number;
  rank: number;
}

export class GameManager implements RuntimeModule, Listener {
  private game: WasmGame | null;
  private waitingForInput: boolean;
  private _selected: SelectedPiece | null;
  private _board: Board;

  public constructor() {
    this.game = null;
    this._selected = null;
    this.waitingForInput = false;
    this._board = new Board();
  }

  public init(): boolean {
    runtime.messageQueue.subscribe(EVENT_MAP.NEW_GAME, this);
    runtime.messageQueue.subscribe(EVENT_MAP.REQUEST_INPUT, this);
    runtime.messageQueue.subscribe(EVENT_MAP.ANIMATION_DONE, this);
    this.game = new WasmGame();
    return this.newgame(undefined);
  }

  public newgame(fen: string | undefined): boolean {
    if (!this.game) {
      console.error('Game not initialized. Cannot restart.');
      return false;
    }

    fen = fen || (document.getElementById('fenInput') as HTMLInputElement)?.value || DEFAULT_FEN;
    this.waitingForInput = false;
    try {
      const isAiPlayer = (player: string) => {
        const element = document.querySelector(`input[name="${player}"]:checked`);
        if (element && element instanceof HTMLInputElement) {
          return element.value === 'computer';
        }
        return false;
      };

      console.log(`Starting a new game >>>>>>`);

      this.game.reset_game(fen, !isAiPlayer('player1'), !isAiPlayer('player2'));
      this.updateBoard();

      runtime.messageQueue.emit({ event: EVENT_MAP.REQUEST_INPUT })
      return true;
    } catch (e) {
      console.error(`Error parsing FEN '${fen}': ${e}`);
      return false;
    }
  }

  public get board(): Board {
    return this._board;
  }

  public handleMessage(event: string) {
    switch (event) {
      case EVENT_MAP.NEW_GAME: {
        this.newgame(undefined);
      } break;
      case EVENT_MAP.REQUEST_INPUT: {
        this.onRequestPlayerInput();
      } break;
      case EVENT_MAP.ANIMATION_DONE: {
        runtime.messageQueue.emit({ event: EVENT_MAP.REQUEST_INPUT });
      } break;
      default: break;
    }
  }

  private onRequestPlayerInput() {
    if (!this.game?.game_over()) {
      this.waitingForInput = true;
      return;
    }

    alert('Game over!');
    this.newgame(DEFAULT_FEN);
  }

  private updateBoard() {
    if (!this.game) {
      return;
    }

    if (this._board.set(this.game.fen(), this.game.get_legal_moves())) {
      console.log(this.game.debug_string());
    }
  }

  public get selectedPiece() {
    return this._selected;
  }

  public tick() {
    if (!this.game || !this._board) {
      return;
    }

    if (this.waitingForInput) {
      const move = this.game.get_move();
      if (move) {
        const wasmMove = this.game.make_move(move);
        if (wasmMove && !wasmMove.is_none()) {
          runtime.messageQueue.emit({ event: EVENT_MAP.MOVE, payload: wasmMove });
          this.waitingForInput = false;
          this.updateBoard();
        } else {
          runtime.messageQueue.emit({ event: EVENT_MAP.REQUEST_INPUT });
        }
      }
    }
  }

  public injectMove(move: string) {
    if (!this.game) {
      console.error('Game not initialized. Cannot inject move.');
      return;
    }

    this.game.inject_move(move);
  }
}
