import { DEFAULT_FEN } from './constants';
import * as BitboardX from '../../pkg/bitboard_x';
import { RuntimeModule, runtime } from './runtime';
import { Listener, Message } from './message-queue';
import { Board } from './board';

export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number;
  rank: number;
}


export class GameManager implements RuntimeModule, Listener {
  private game: BitboardX.WasmGame | null;
  private _selected: SelectedPiece | null;
  private canvas: HTMLCanvasElement | null;
  private _board: Board;
  private waitingForInput: boolean;

  public constructor() {
    this.game = null;
    this._selected = null;
    this.canvas = null;
    this.waitingForInput = false;
    this._board = new Board();
  }

  public newgame(fen: string | undefined): boolean {
    if (!this.game) {
      console.error('Game not initialized. Cannot restart.');
      return false;
    }

    fen = fen || (document.getElementById('fenInput') as HTMLInputElement).value || DEFAULT_FEN;
    this.waitingForInput = false;
    try {
      const isPlayerHuman = (player: string) => {
        const element = document.querySelector(`input[name="${player}"]:checked`);
        if (element && element instanceof HTMLInputElement) {
          return element.value === 'human';
        }
        return false;
      };

      console.log(`Starting a new game >>>>>>`);

      this.canvas = runtime.display.canvas;
      this.game.reset_game(fen, isPlayerHuman('player1'), isPlayerHuman('player2'));
      this.updateBoard();

      runtime.messageQueue.emit(Message.REQUEST_PLAYER_INPUT)
      return true;
    } catch (e) {
      console.error(`Error parsing FEN '${fen}': ${e}`);
      return false;
    }
  }

  public get board(): Board {
    return this._board;
  }

  public handleMessage(message: string) {
    const [eventType] = message.split(':');
    switch (eventType) {
      case Message.NEW_GAME: {
        this.newgame(undefined);
      } break;
      case Message.REQUEST_PLAYER_INPUT: {
        this.onRequestPlayerInput();
      } break;
      case Message.ANIMATION_DONE: {
        runtime.messageQueue.emit(Message.REQUEST_PLAYER_INPUT);
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

  public init(): boolean {
    runtime.messageQueue.subscribe(Message.NEW_GAME, this);
    runtime.messageQueue.subscribe(Message.REQUEST_PLAYER_INPUT, this);
    runtime.messageQueue.subscribe(Message.ANIMATION_DONE, this);
    this.game = new BitboardX.WasmGame();
    return this.newgame(undefined);
  }

  public tick() {
    if (!this.game || !this._board) {
      return;
    }

    if (this.waitingForInput) {
      const move = this.game.get_move();
      if (move) {
        const isMoveValid = this.game.make_move(move);
        if (isMoveValid) {
          runtime.messageQueue.emit(`${Message.MOVE}:${move}`);
          this.waitingForInput = false;
          this.updateBoard();
        } else {
          runtime.messageQueue.emit(Message.REQUEST_PLAYER_INPUT);
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
