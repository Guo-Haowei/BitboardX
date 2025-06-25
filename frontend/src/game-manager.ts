import { DEFAULT_FEN } from './constants';
import { RuntimeModule, runtime } from './runtime';
import { Listener, EVENT_MAP } from './message-queue';
import { Board } from './board';
import { WasmPosition, WasmEngine } from '../../bitboard_x/pkg/bitboard_x';

export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number;
  rank: number;
}

interface Player {
  name: string;
  getMove(history: string): Promise<string>; // returns UCI move, e.g. "e2e4"
}

class BotPlayer implements Player {
  name: string;
  private engine: WasmEngine

  constructor() {
    this.name = 'Bot';
    this.engine = new WasmEngine();
  }

  async getMove(history: string): Promise<string> {
    const searchDepth = 5; // Example depth, can be adjusted
    this.engine.set_position(history);

    const bestMove = this.engine.best_move(searchDepth);

    if (bestMove) {
      return bestMove.to_string();
    }
    throw new Error("No best move found");

  }
}

class ChessBoard {
  position: WasmPosition;
  legalMoves: string[] = [];
  private _history: string;

  constructor(fen: string | undefined) {
    this.position = new WasmPosition(fen || DEFAULT_FEN);
    const initialPosition = fen ? `fen ${fen}` : 'startpos';
    this._history = `position ${initialPosition} moves`;
    this.legalMoves = this.position.legal_moves();
  }

  isGameOver(): boolean {
    return this.legalMoves.length === 0;
  }

  turn(): number {
    return this.position.turn();
  }

  makeMove(move: string): boolean {
    if (!this.position.make_move(move)) {
      // console.warn(`Invalid move attempted: ${move}`);
      return false;
    }

    this._history += ` ${move}`;
    return true;
  }

  get history(): string {
    return this._history;
  }
}

class GameController {
  private players: Player[];
  board: ChessBoard;

  constructor(white: Player, black: Player, fen: string | undefined) {
    this.board = new ChessBoard(fen || DEFAULT_FEN);
    this.players = [white, black];
  }

  public activePlayer(): Player {
    return this.players[this.board.turn()];
  }

  public isGameOver(): boolean {
    return this.board.isGameOver();
  }

  async start(): Promise<void> {
    while (!this.board.isGameOver()) {
      const activePlayer = this.activePlayer();

      const move = await activePlayer.getMove(this.board.history);

      if (!this.board.makeMove(move)) {
        // optionally throw or notify
        continue;
      }
    }
  }
}

export class GameManager implements RuntimeModule, Listener {
  private controller: GameController | null = null;
  private waitingForInput: boolean;
  private _selected: SelectedPiece | null;
  private _board: Board;

  public constructor() {
    this._selected = null;
    this.waitingForInput = false;
    this._board = new Board();
  }

  public init(): boolean {
    runtime.messageQueue.subscribe(EVENT_MAP.NEW_GAME, this);
    runtime.messageQueue.subscribe(EVENT_MAP.REQUEST_INPUT, this);
    runtime.messageQueue.subscribe(EVENT_MAP.ANIMATION_DONE, this);
    return this.newgame(undefined);
  }

  public newgame(fen: string | undefined): boolean {
    fen = fen || (document.getElementById('fenInput') as HTMLInputElement)?.value || DEFAULT_FEN;
    this.controller = new GameController(
      new BotPlayer(), // White player
      new BotPlayer(), // Black player
      fen
    );

    this.waitingForInput = false;
    // try {
    //   const isAiPlayer = (player: string) => {
    //     const element = document.querySelector(`input[name="${player}"]:checked`);
    //     if (element && element instanceof HTMLInputElement) {
    //       return element.value === 'computer';
    //     }
    //     return false;
    //   };

    //   console.log(`Starting a new game >>>>>>`);

    //   this.game.reset_game(fen, !isAiPlayer('player1'), !isAiPlayer('player2'));
    //   return true;
    // } catch (e) {
    //   console.error(`Error parsing FEN '${fen}': ${e}`);
    //   return false;
    // }
    this.updateBoard();

    runtime.messageQueue.emit({ event: EVENT_MAP.REQUEST_INPUT })
    return true;
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
    if (!this.controller?.isGameOver()) {
      this.waitingForInput = true;
      return;
    }

    alert('Game over!');
    this.newgame(DEFAULT_FEN);
  }

  private updateBoard() {
    const board = this.controller?.board;
    if (!board)
      return;

    if (this._board.set(board.position.fen(), board.legalMoves)) {
      console.log(board.position.debug_string());
    }
  }

  public get selectedPiece() {
    return this._selected;
  }

  async tick() {
    const board = this.controller?.board;
    if (!board)
      return;

    if (this.waitingForInput) {
      const move = await this.controller?.activePlayer().getMove(board.history)!;

      if (board.makeMove(move)) {
        // optionally throw or notify
        runtime.messageQueue.emit({ event: EVENT_MAP.MOVE, payload: move });
        this.waitingForInput = false;
        this.updateBoard();
      } else {
        runtime.messageQueue.emit({ event: EVENT_MAP.REQUEST_INPUT });
      }
    }
  }
}
