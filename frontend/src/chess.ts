import init from '../../bitboard_x/pkg/bitboard_x';
import { BOARD_SIZE, COLORS, PIECE_SYMBOLS } from './constants';
import { isLowerCase, fileRankToSquare } from './utils';
import { DEFAULT_FEN } from './constants';
import { WasmPosition, WasmEngine, WasmMove, name } from '../../bitboard_x/pkg/bitboard_x';

export const PIECE_RES = new Map<string, HTMLImageElement>();
const PIECE_CODES = ['wP', 'wN', 'wB', 'wR', 'wQ', 'wK', 'bP', 'bN', 'bB', 'bR', 'bQ', 'bK'];

let renderer: Renderer | null = null;
let engine: WasmEngine | null = null;
let uiCountroller: UIController | null = null;
let gameController: GameController | null = null;

export function createGame(white: Player, black: Player, fen?: string) {

  gameController = new GameController(
    white,
    black,
    fen,
  );

  return gameController;
}

async function loadImage(code: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    const url = `https://lichess1.org/assets/piece/cburnett/${code}.svg`;
    img.src = url;
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`Failed to load: ${url}`));
  });
}

export async function initialize(callback: () => void) {
  await init();

  Promise.all(PIECE_CODES.map(loadImage))
    .then(images => {
      console.log("✅ All assets loaded");
      images.forEach((img, index) => {
        const code = PIECE_CODES[index];
        const color = code[0];
        const piece = color === 'w' ? code[1] : code[1].toLowerCase();
        PIECE_RES.set(piece, img);
      });

      console.log(`Initializing engine ${name()}`);
      engine = new WasmEngine();
      renderer = new Renderer();
      uiCountroller = new UIController(renderer.canvas);

      callback();
    })
    .catch(err => {
      console.error("❌ One or more images failed to load:", err);
    });
}

// ---------------------------- Renderer -----------------------------------

const GREEN_COLOR = 'rgba(0, 200, 0, 0.5)';
const RED_COLOR = 'rgba(200, 0, 0, 0.5)';
const YELLOW_COLOR_1 = 'rgba(150, 150, 0, 0.5)';
const YELLOW_COLOR_2 = 'rgba(200, 200, 0, 0.5)';

export class Renderer {
  private ctx: CanvasRenderingContext2D;
  private images: Map<string, HTMLImageElement>;
  canvas: HTMLCanvasElement;
  tileSize = 0;

  public constructor() {
    const canvas = document.getElementById('chessCanvas') as HTMLCanvasElement;
    canvas.tabIndex = 0;
    canvas.style.margin = '20px auto';

    this.canvas = canvas;
    const minSize = 800;
    this.resizeCanvas(canvas, minSize);

    this.ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
    if (!this.ctx) {
      throw new Error("Failed to get canvas context");
    }

    this.ctx.font = '40px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';

    this.images = PIECE_RES;
  }

  resizeCanvas(canvas: HTMLCanvasElement, minSize = 200) {

    let size = Math.min(canvas.clientWidth, canvas.clientHeight);
    size = Math.max(size, minSize);
    canvas.width = size;
    canvas.height = size;

    this.tileSize = size / (BOARD_SIZE + 1);
    console.log(`tile size is ${this.tileSize}`);
  }

  async draw(board?: ChessBoard) {
    const { ctx, canvas } = this;
    board = board || gameController?.board;
    if (board) {
      ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight);
      ctx.font = `${this.tileSize / 2}px Arial`
      this.drawBoard(board);
      this.drawPieces(board);
    }
  }

  private fillSquare(col: number, row: number, color: string) {
    if (!this.ctx) {
      return;
    }
    const { tileSize } = this;
    this.ctx.fillStyle = color;
    this.ctx.fillRect(col * tileSize, row * tileSize, tileSize, tileSize);
  }

  private drawBoard(board: ChessBoard) {
    const { ctx } = this;
    if (!ctx) {
      return;
    }

    const { tileSize } = this;

    const selected = uiCountroller?.selected || '';
    const legalMoves = board.legalMovesMap.get(selected);

    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        const color = ((row + col) % 2 === 0 ? COLORS.light : COLORS.dark);
        this.fillSquare(col, row, color);

        const sq = fileRankToSquare(col, 7 - row);
        if (sq === selected) {
          this.fillSquare(col, row, RED_COLOR);
        } else if (legalMoves && legalMoves.has(sq)) {
          this.fillSquare(col, row, GREEN_COLOR);
        }

        const lastMove = board.history.length > 0 ? board.history[board.history.length - 1] : null;
        if (lastMove) {
          const src = lastMove.src_sq();
          const dst = lastMove.dst_sq();
          if (sq === src) {
            this.fillSquare(col, row, YELLOW_COLOR_1);
          } else if (sq === dst) {
            this.fillSquare(col, row, YELLOW_COLOR_2);
          }
        }
      }
    }

    // draw file labels
    ctx.fillStyle = 'black';
    for (let file = 0; file < BOARD_SIZE; ++file) {
      const x = file * tileSize + tileSize / 2;
      const y = BOARD_SIZE * tileSize + tileSize / 2;
      ctx.fillText(String.fromCharCode(97 + file).toString(), x, y);
    }

    // draw rank labels
    for (let row = 0; row < BOARD_SIZE; row++) {
      const x = BOARD_SIZE * tileSize + tileSize / 2;
      const y = row * tileSize + tileSize / 2;
      ctx.fillText((8 - row).toString(), x, y);
    }
  }

  private drawPiece(piece: string, x: number, y: number) {
    const { tileSize } = this;
    const img = this.images.get(piece);
    if (img) {
      const half = tileSize / 2;
      this.ctx.drawImage(img, x - half, y - half, tileSize, tileSize);
    } else {
      this.ctx.fillStyle = isLowerCase(piece) ? 'black' : 'white';
      this.ctx.fillText(PIECE_SYMBOLS[piece], x, y);
    }
  }

  private drawPieces(board: ChessBoard) {
    const animated = new Set<number>();
    const { tileSize } = this;

    // for (const animation of animations) {
    //   const { piece, dstFile, dstRank } = animation;
    //   const idx = dstFile + dstRank * BOARD_SIZE;
    //   animated.add(idx);
    //   const x = animation.x * tileSize + tileSize / 2;
    //   const y = animation.y * tileSize + tileSize / 2;
    //   this.drawPiece(piece, x, y);
    // }

    const boardString = board.position.board_string();

    for (let row = 0; row < BOARD_SIZE; ++row) {
      for (let col = 0; col < BOARD_SIZE; ++col) {
        const idx = (7 - row) * BOARD_SIZE + col;
        const piece = boardString[idx];
        if (piece === '.') {
          continue;
        }
        if (animated.has(idx)) {
          continue; // Skip pieces that are currently animated
        }
        const x = col * tileSize + tileSize / 2;
        const y = row * tileSize + tileSize / 2;
        this.drawPiece(piece, x, y);
      }
    }
  }
}

// ---------------------------- Game Controller -------------------------------


export interface SelectedPiece {
  piece: string;
  x: number;
  y: number;
  file: number;
  rank: number;
}

export interface Player {
  getMove(history: string): Promise<string>; // returns UCI move, e.g. "e2e4"
}

export class BotPlayer implements Player {
  async getMove(history: string): Promise<string> {
    const searchDepth = 5;

    engine?.set_position(history);

    const bestMove = engine?.best_move(searchDepth);

    if (bestMove) {
      return bestMove;
    }
    throw new Error("No best move found");
  }
}

class UIController {
  selected: string | null = null;
  private moves = new Set<string>();
  private resolveMove: ((move: string) => void) | null = null;

  constructor(private canvas: HTMLCanvasElement) {
    canvas.addEventListener('mouseup', this.onClick);
  }

  waitForPlayerMove(): Promise<string> {
    return new Promise((resolve) => {
      this.resolveMove = resolve;
      this.selected = null;
    });
  }

  private onClick = (event: MouseEvent) => {
    if (!this.resolveMove) return;
    const x = event.offsetX;
    const y = event.offsetY;
    const tileSize = renderer?.tileSize || 0;
    const file = Math.floor(x / tileSize);
    const rank = 7 - Math.floor(y / tileSize);
    const square = fileRankToSquare(file, rank);

    if (!this.selected) {
      if (gameController?.board.legalMovesMap.has(square)) {
        this.selected = square;
      } else {
        return;
      }
    } else {
      if (gameController?.board.legalMovesMap.get(this.selected)?.has(square)) {
        const resolve = this.resolveMove;
        this.resolveMove = null;
        resolve(`${this.selected}${square}`);
      } else {
        this.selected = null; // deselect if not a legal move
      }
    }

    renderer?.draw();
  }
};

export class UIPlayer implements Player {
  getMove(): Promise<string> {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    return uiCountroller!.waitForPlayerMove();
  }
}

export class ChessBoard {
  position: WasmPosition;
  legalMoves: string[] = [];
  legalMovesMap = new Map<string, Set<string>>();
  initialPos: string;
  history: WasmMove[];

  constructor(fen: string | undefined) {
    this.position = new WasmPosition(fen || DEFAULT_FEN);

    this.initialPos = fen ? `fen ${fen}` : 'startpos';
    this.history = [];
    this.updateLegalMoves();
  }

  updateLegalMoves() {
    this.legalMoves = this.position.legal_moves();
    this.legalMovesMap.clear();

    for (const move of this.legalMoves) {
      const src = move.slice(0, 2); // e.g. "e2"
      const dst = move.slice(2, 4); // e.g. "e4"
      if (!this.legalMovesMap.has(src)) {
        this.legalMovesMap.set(src, new Set());
      }
      this.legalMovesMap.get(src)?.add(dst);
    }
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

      this.updateLegalMoves();
    }

    return move;
  }
}

class GameController {
  private players: Player[];
  private isRunning = false;
  board: ChessBoard;

  constructor(white: Player, black: Player, fen?: string) {
    this.board = new ChessBoard(fen);
    this.players = [white, black];
    renderer?.draw(this.board);
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
    this.isRunning = true;
    this.step();
  }

  stop() {
    this.isRunning = false;
  }

  private step = async () => {
    if (!this.isRunning) {
      return;
    }

    const activePlayer = this.activePlayer();

    const moveStr = await activePlayer.getMove(this.uciPosition());

    const move = this.board.makeMove(moveStr);

    if (move) {
      await renderer?.draw();
    }

    setTimeout(() => {
      requestAnimationFrame(this.step);
    }, 200); // controls pace between moves
  };
}