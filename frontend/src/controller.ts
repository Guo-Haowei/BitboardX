import { DEFAULT_FEN } from './constants';
import { WasmPosition, WasmEngine, WasmMove } from '../../bitboard_x/pkg/bitboard_x';
import { runtime } from './runtime';

export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number;
  rank: number;
}

export interface Player {
  name: string;
  getMove(history: string): Promise<string>; // returns UCI move, e.g. "e2e4"
}

export class BotPlayer implements Player {
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

export class ChessBoard {
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

export class GameController {
  private players: Player[];
  board: ChessBoard;

  constructor(white: Player, black: Player, fen?: string) {
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

      const moveStr = await activePlayer.getMove(this.uciPosition());
      console.log(moveStr);

      const move = this.board.makeMove(moveStr);

      if (!move) {
        // optionally throw or notify
        continue;
      }

      runtime.renderer.draw(this.board);
      await this.sleep(200); // prevent blocking
    }
  }

  sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// fen = fen || (document.getElementById('fenInput') as HTMLInputElement)?.value || DEFAULT_FEN;
// this.controller = new GameController(
//   new BotPlayer(), // White player
//   new BotPlayer(), // Black player
//   fen
// );