import { DEFAULT_FEN } from './constants';
import * as BitboardX from '../../pkg/bitboard_x';
import { RuntimeModule, runtime } from './runtime';
import { messageQueue, Listener, Message } from './message-queue';
import { Board } from './board';

export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number; // from file
  rank: number; // from rank
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
    messageQueue.subscribe(Message.NEW_GAME, this);
    messageQueue.subscribe(Message.REQUEST_PLAYER_INPUT, this);
    messageQueue.subscribe(Message.ANIMATION_DONE, this);
  }

  public newgame(fen: string): boolean {
    if (!this.game) {
      console.error('Game not initialized. Cannot restart.');
      return false;
    }

    this.waitingForInput = false;

    // const fen = (document.getElementById('fenInput') as HTMLInputElement).value || DEFAULT_FEN;
    try {
      const isPlayerHuman = (player: string) => {
        const element = document.querySelector(`input[name="${player}"]:checked`);
        if (element && element instanceof HTMLInputElement) {
          return element.value === 'human';
        }
        return false;
      };

      this.canvas = runtime.display.canvas;
      this.game.reset_game(fen, isPlayerHuman('player1'), isPlayerHuman('player2'));
      this.updateBoard();

      messageQueue.emit(Message.REQUEST_PLAYER_INPUT)
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
    const [eventType, ...payload] = message.split(':');
    switch (eventType) {
      case Message.NEW_GAME: {
        this.newgame(payload[0]);
      } break;
      case Message.REQUEST_PLAYER_INPUT: {
        this.waitingForInput = true;
      } break;
      case Message.ANIMATION_DONE: {
        messageQueue.emit(Message.REQUEST_PLAYER_INPUT);
      } break;
      default: break;
    }
  }

  private updateBoard() {
    if (!this.game) {
      return;
    }
    if (this._board.set(this.game.to_board_string(), this.game.get_legal_moves())) {
      console.log(this.game.debug_string());
    }
  }

  public getName(): string {
    return 'Game';
  }

  public get selectedPiece() {
    return this._selected;
  }

  public init(): boolean {
    this.game = new BitboardX.WasmGame();
    return this.newgame(DEFAULT_FEN);
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
          messageQueue.emit(`${Message.MOVE}:${move}`);
          this.waitingForInput = false;
          this.updateBoard();
        } else {
          messageQueue.emit(Message.REQUEST_PLAYER_INPUT);
        }
      }
    }

    // @TODO: better game result handling
    if (this.game?.game_over()) {
      alert('Game over!');
      // this.restart();
      return;
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
