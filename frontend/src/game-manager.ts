import { DEFAULT_FEN } from './constants';
import { RuntimeModule, runtime } from './runtime';
import { Listener, EVENT_MAP } from './message-queue';
import { Board } from './board';
import { WasmPosition, WasmEngine, WasmMove } from '../../bitboard_x/pkg/bitboard_x';

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
      return bestMove;
    }
    throw new Error("No best move found");

  }
}

class ChessBoard {
  position: WasmPosition;
  legalMoves: string[] = [];
  initialPos: string;
  history: WasmMove[];

  constructor(fen: string | undefined) {
    this.position = new WasmPosition(fen || DEFAULT_FEN);
    this.legalMoves = this.position.legal_moves();

    this.initialPos = fen ? `fen ${fen}` : 'startpos';
    this.history = [];
  }

  isGameOver(): boolean {
    return this.legalMoves.length === 0;
  }

  turn(): number {
    return this.position.turn();
  }

  makeMove(move_str: string) {
    const move = this.position.make_move(move_str);
    if (move) {
      this.history.push(move);
    }

    return move;
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

  public uciPosition(): string {
    const { history } = this.board;
    const moves = history.length > 0 ? `moves ${history.map(mv => mv.to_string()).join(' ')}` : '';
    const uci = `position ${this.board.initialPos} ${moves}`;
    return uci;
  }

  async start(): Promise<void> {
    while (!this.board.isGameOver()) {
      const activePlayer = this.activePlayer();

      const move = await activePlayer.getMove(this.uciPosition());

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
    const { controller } = this;
    if (!controller) {
      return;
    }

    if (this.waitingForInput) {

      const position = controller.uciPosition();
      console.log(position);
      const move_str = await controller.activePlayer().getMove(position);

      const move = controller.board.makeMove(move_str);

      if (move) {
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
