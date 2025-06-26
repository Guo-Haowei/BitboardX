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
let board: ChessBoard | null = null;

// ---------------------------- Initialization -------------------------------
export function createGame(white: Player, black: Player, fen?: string) {
  board = new ChessBoard(fen || DEFAULT_FEN);

  gameController = new GameController(
    white,
    black,
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

interface Config {
  canvas: HTMLCanvasElement;
}

export async function initialize(config: Config, callback: () => void) {
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

      const { canvas } = config;

      const viewport = new Viewport(canvas);
      renderer = new Renderer(viewport);
      uiCountroller = new UIController(viewport);

      console.log(`✅ Initializing engine ${name()}`);
      engine = new WasmEngine();

      callback();
    })
    .catch(err => {
      console.error("❌ One or more images failed to load:", err);
    });
}

// ---------------------------- Chess Board Wrapper -----------------------------
class ChessBoard {
  private position: WasmPosition;
  private initialPos: string;

  private _boardString = '';
  private _legalMoves: string[] = [];
  private _legalMovesMap = new Map<string, Set<string>>();

  private history: WasmMove[];

  constructor(fen: string) {
    this.position = new WasmPosition(fen);

    this.initialPos = `fen ${fen}`;
    this.history = [];

    this.updateInternal();
  }

  get boardString() {
    return this._boardString;
  }

  get legalMovesMap() {
    return this._legalMovesMap;
  }

  public uciPosition(): string {
    const { history, initialPos } = this;
    const moves = history.length > 0 ? `moves ${history.map(mv => mv.to_string()).join(' ')}` : '';
    const uci = `position ${initialPos} ${moves}`;
    return uci;
  }

  public lastMove(): WasmMove | null {
    return this.history.length > 0 ? this.history[this.history.length - 1] : null;
  }

  private updateInternal() {
    // board string
    this._boardString = this.position.board_string();
    // legal moves
    this._legalMoves = this.position.legal_moves();
    this._legalMovesMap.clear();

    for (const move of this._legalMoves) {
      const src = move.slice(0, 2); // e.g. "e2"
      const dst = move.slice(2, 4); // e.g. "e4"
      if (!this._legalMovesMap.has(src)) {
        this._legalMovesMap.set(src, new Set());
      }
      this._legalMovesMap.get(src)?.add(dst);
    }
  }

  isGameOver(): boolean {
    return this._legalMoves.length === 0;
  }

  turn(): number {
    return this.position.turn();
  }

  makeMove(move_str: string) {
    const move = this.position.make_move(move_str);
    if (move) {
      this.history.push(move);

      this.updateInternal();
    }

    return move;
  }

  getPieceAt(square: string) {
    const index = squareStringToIndex(square);
    return this._boardString[index];
  }
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

  async draw() {
    const { ctx, viewport } = this;
    const { width, height } = viewport.canvas;
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

    const lastMove = board.lastMove();
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
    const boardString = board.boardString;
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

class UIController {
  selected: string | null = null;
  x = -1;
  y = -1;
  private selectingPromotion = false;
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
    const square = this.viewport.screenToSquare(event.offsetX, event.offsetY);

    if (board!.legalMovesMap.has(square)) {
      this.selected = square;
    }

    renderer!.draw();
  }

  private onMouseMove = (event: MouseEvent) => {
    if (this.selectingPromotion) return;
    this.x = event.offsetX;
    this.y = event.offsetY;
    renderer!.draw();
  }

  private onMouseUp = async (event: MouseEvent) => {
    if (!this.resolveMove || !this.selected) return;
    const square = this.viewport.screenToSquare(event.offsetX, event.offsetY);

    if (board!.legalMovesMap.get(this.selected)?.has(square)) {
      let move = `${this.selected}${square}`;

      const piece = board!.getPieceAt(this.selected);

      let promotion = null;
      if (piece === 'P' && square[1] === '8') {
        promotion = true;
      } else if (piece === 'p' && square[1] === '1') {
        promotion = false;
      }

      if (promotion !== null) {
        this.selected = null;
        this.selectingPromotion = true;
        const promotionPiece = await this.waitForPromotionSelection(promotion, event);
        this.selectingPromotion = false;
        move += promotionPiece;
      }

      const resolve = this.resolveMove;
      this.resolveMove = null;
      resolve(move);
    }

    this.selected = null;
    renderer!.draw();
  }

  private waitForPromotionSelection(isWhite: boolean, event: MouseEvent): Promise<string> {
    return new Promise((resolve) => {
      const container = document.createElement('div');
      container.id = 'promotion-dialog';
      container.style.position = 'absolute';
      container.style.left = `${event.clientX}px`;
      container.style.top = `${event.clientY}px`;
      container.style.display = 'flex';
      container.style.gap = '8px';
      container.style.zIndex = '9999';
      container.style.background = 'rgba(255, 255, 255, 0.95)';
      container.style.padding = '6px';
      container.style.border = '1px solid #ccc';
      container.style.borderRadius = '4px';
      container.style.boxShadow = '0 2px 6px rgba(0,0,0,0.3)';

      const pieces = ['Q', 'R', 'B', 'N'];

      for (let piece of pieces) {
        if (!isWhite) {
          piece = piece.toLowerCase();
        }

        const option = document.createElement('div');
        option.style.width = '64px';
        option.style.height = '64px';
        option.style.cursor = 'pointer';
        option.style.backgroundImage = `url(${PIECE_RES.get(piece)?.src})`;
        option.style.backgroundSize = 'contain';
        option.style.backgroundRepeat = 'no-repeat';
        option.style.backgroundPosition = 'center';

        option.onclick = () => {
          document.body.removeChild(container);
          resolve(piece);
        };

        container.appendChild(option);
      }

      document.body.appendChild(container);
    });
  }
};

// ---------------------------- Game Controller -------------------------------
class GameController {
  private players: Player[];
  private isRunning = false;

  constructor(white: Player, black: Player) {
    this.players = [white, black];
    renderer?.draw();
  }

  public activePlayer(): Player {
    return this.players[board!.turn()];
  }

  public isGameOver(): boolean {
    return board!.isGameOver();
  }

  async start(): Promise<void> {
    this.isRunning = true;
    this.step();
  }

  stop() {
    this.isRunning = false;
  }

  private step = async () => {
    if (!this.isRunning) return;

    if (this.isGameOver()) {
      alert("Game over!");
      return;
    }

    const activePlayer = this.activePlayer();

    const moveStr = await activePlayer.getMove(board!.uciPosition());

    const move = board!.makeMove(moveStr);

    if (move) {
      await renderer!.draw();
    }

    setTimeout(() => {
      requestAnimationFrame(this.step);
    }, 200); // controls pace between moves
  };
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

export class UIPlayer implements Player {
  getMove(): Promise<string> {

    return uiCountroller!.waitForPlayerMove();
  }
}

// ---------------------------- Utils ------------------------------------
function fileRankToSquare(file: number, rank: number) {
  return `${String.fromCharCode(97 + file)}${rank + 1}`;
};

function squareToFileRank(square: number): [number, number] {
  return [square % BOARD_SIZE, Math.floor(square / BOARD_SIZE)];
}

function squareStringToIndex(square: string): number {
  const file = square.charCodeAt(0) - 97; // 'a' is 97
  const rank = parseInt(square[1], 10) - 1; // '1' is 1
  return rank * BOARD_SIZE + file;
}
