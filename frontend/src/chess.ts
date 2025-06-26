/* eslint-disable @typescript-eslint/no-non-null-assertion */
import init from '../../bitboard_x/pkg/bitboard_x';
import { WasmPosition, WasmEngine, WasmMove, name } from '../../bitboard_x/pkg/bitboard_x';

const PIECE_RES = new Map<string, HTMLImageElement>();
const PIECE_CODES = ['wP', 'wN', 'wB', 'wR', 'wQ', 'wK', 'bP', 'bN', 'bB', 'bR', 'bQ', 'bK'];

const BOARD_SIZE = 8;
const DEFAULT_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

// @TODO: allow user to configure the colors
const GREEN_COLOR = 'rgba(0, 200, 0, 0.5)';
const RED_COLOR = 'rgba(200, 0, 0, 0.5)';
const YELLOW_COLOR = 'rgba(200, 200, 0, 0.5)';
const LIGHT_SQUARE_COLOR = 'rgba(240, 217, 181, 1)';
const DARK_SQUARE_COLOR = 'rgba(181, 136, 99, 1)';

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

      const canvas = document.getElementById('chessCanvas') as HTMLCanvasElement;
      canvas.tabIndex = 0;
      canvas.style.margin = '20px auto';

      const viewport = new Viewport(canvas);
      renderer = new Renderer(viewport);
      uiCountroller = new UIController(viewport);

      console.log(`Initializing engine ${name()}`);
      engine = new WasmEngine();

      callback();
    })
    .catch(err => {
      console.error("❌ One or more images failed to load:", err);
    });
}

// ---------------------------- GUI and Renderer -------------------------------
class Viewport {
  squareSize = 0;
  canvas: HTMLCanvasElement;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    window.addEventListener('resize', () => {
      this.resize();
    });
    this.resize();
  }

  resize() {
    const { clientWidth, clientHeight } = this.canvas;
    const minSize = 200;
    const size = Math.max(minSize, Math.min(clientWidth, clientHeight));
    this.canvas.width = size;
    this.canvas.height = size;
    this.squareSize = size / BOARD_SIZE;
  }

  screenToSquare(x: number, y: number): string {
    const file = Math.floor(x / this.squareSize);
    const rank = 7 - Math.floor(y / this.squareSize);
    return fileRankToSquare(file, rank);
  }
}

class Renderer {
  private ctx: CanvasRenderingContext2D;
  private viewport: Viewport;

  private get squareSize() {
    return this.viewport.squareSize;
  }

  public constructor(viewport: Viewport) {
    this.ctx = viewport.canvas.getContext('2d') as CanvasRenderingContext2D;
    if (!this.ctx) {
      throw new Error("Failed to get canvas context");
    }

    this.viewport = viewport;
  }

  async draw(board?: ChessBoard) {
    const { ctx, viewport } = this;
    const { width, height } = viewport.canvas;
    board = board || gameController?.board;
    if (board) {
      ctx.clearRect(0, 0, width, height);
      this.drawBoard(board);
      this.drawPieces(board);
    }
  }

  private fillSquare(col: number, row: number, color: string) {
    const { squareSize, ctx } = this;
    ctx.fillStyle = color;
    ctx.fillRect(col * squareSize, row * squareSize, squareSize, squareSize);
  }

  private drawBoard(board: ChessBoard) {
    const { ctx, squareSize } = this;

    const selected = uiCountroller?.selected || '';
    const legalMoves = board.legalMovesMap.get(selected);

    const lastMove = board.history.length > 0 ? board.history[board.history.length - 1] : null;
    for (let idx = 0; idx < BOARD_SIZE * BOARD_SIZE; ++idx) {
      const [file, rank] = squareToFileRank(idx);
      const sq = fileRankToSquare(file, rank);

      const row = 7 - rank; // flip the rank for rendering
      this.fillSquare(file, row, (row + file) % 2 === 0 ? LIGHT_SQUARE_COLOR : DARK_SQUARE_COLOR);
      if (sq === selected) {
        this.fillSquare(file, row, RED_COLOR);
      } else if (legalMoves && legalMoves.has(sq)) {
        this.fillSquare(file, row, GREEN_COLOR);
      }
      if (lastMove && (sq === lastMove.src_sq() || sq === lastMove.dst_sq())) {
        this.fillSquare(file, row, YELLOW_COLOR);
      }
    }

    const fontSize = Math.floor(squareSize / 4);
    ctx.font = `${fontSize}px Arial`;
    ctx.textAlign = 'center';
    for (let i = 0; i < BOARD_SIZE; ++i) {
      const color = (i % 2 === 0 ? LIGHT_SQUARE_COLOR : DARK_SQUARE_COLOR);
      const fileStr = String.fromCharCode(97 + i); // 'a' + i
      const rankStr = (i + 1).toString();
      ctx.fillStyle = color;
      let x = (i + 0.88) * squareSize;
      let y = 7.93 * squareSize;
      ctx.fillText(fileStr, x, y);
      x = 0.1 * squareSize;
      y = (7.3 - i) * squareSize;
      ctx.fillText(rankStr, x, y);
    }
  }

  private drawPiece(piece: string, x: number, y: number) {
    // make it a bit smaller than the tile size so it doesn't block the file/rank labels
    const pieceSize = this.squareSize * 0.86;
    const half = pieceSize / 2;
    const img = PIECE_RES.get(piece);
    if (img) this.ctx.drawImage(img, x - half, y - half, pieceSize, pieceSize);
  }

  private drawPieces(board: ChessBoard) {
    const { squareSize } = this;
    const boardString = board.position.board_string();
    const { selected, x, y } = uiCountroller!;
    for (let idx = 0; idx < 64; ++idx) {
      const piece = boardString[idx];
      if (piece === '.') continue;
      const [file, rank] = squareToFileRank(idx);
      const screenX = file * squareSize + squareSize / 2;
      const screenY = (7 - rank) * squareSize + squareSize / 2;
      if (fileRankToSquare(file, rank) === selected) {
        this.drawPiece(piece, x, y);
      } else {
        this.drawPiece(piece, screenX, screenY);
      }
    }
  }
}

// ---------------------------- Game Controller -------------------------------
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
  x = -1;
  y = -1;
  private viewport: Viewport;
  private resolveMove: ((move: string) => void) | null = null;

  constructor(viewport: Viewport) {
    viewport.canvas.addEventListener('mousedown', this.onMouseDown);
    viewport.canvas.addEventListener('mousemove', this.onMouseMove);
    viewport.canvas.addEventListener('mouseup', this.onMouseUp);
    this.viewport = viewport;
  }

  waitForPlayerMove(): Promise<string> {
    return new Promise((resolve) => {
      this.resolveMove = resolve;
      this.selected = null;
    });
  }

  private onMouseDown = (event: MouseEvent) => {
    // if (!this.resolveMove) return;
    const square = this.viewport.screenToSquare(event.offsetX, event.offsetY);

    if (gameController?.board.legalMovesMap.has(square)) {
      this.selected = square;
    }

    renderer?.draw();
  }

  private onMouseMove = (event: MouseEvent) => {
    this.x = event.offsetX;
    this.y = event.offsetY;
    renderer?.draw();
  }

  private onMouseUp = (event: MouseEvent) => {
    if (!this.resolveMove || !this.selected) return;
    const square = this.viewport.screenToSquare(event.offsetX, event.offsetY);

    if (gameController?.board.legalMovesMap.get(this.selected)?.has(square)) {
      const resolve = this.resolveMove;
      this.resolveMove = null;
      resolve(`${this.selected}${square}`);
    }

    this.selected = null;
    renderer?.draw();
  }
};

export class UIPlayer implements Player {
  getMove(): Promise<string> {

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

// ---------------------------- Utils ------------------------------------
function fileRankToSquare(file: number, rank: number) {
  return `${String.fromCharCode(97 + file)}${rank + 1}`;
};

function squareToFileRank(square: number): [number, number] {
  return [square % BOARD_SIZE, Math.floor(square / BOARD_SIZE)];
}
