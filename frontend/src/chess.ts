/* eslint-disable @typescript-eslint/no-non-null-assertion */
import init, { WasmGame, WasmEngine, WasmMove, name } from '../../pkg/bitboard_x';
import { Renderer } from './renderer';

// @TODO: move it to renderer because it's bound to renderer,
export const PIECE_RES = new Map<string, HTMLImageElement>();
const PIECE_CODES = ['wP', 'wN', 'wB', 'wR', 'wQ', 'wK', 'bP', 'bN', 'bB', 'bR', 'bQ', 'bK'];

const BOARD_SIZE = 8;
const DEFAULT_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

let renderer: Renderer | null = null;
let engine: WasmEngine | null = null;
let gameController: GameController | null = null;
let board: ChessBoard | null = null;
let controller: GameController | null = null;

// ---------------------------- Initialization -------------------------------
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

      // @TODO: create 2D or 3D
      renderer = new Renderer(canvas);
      renderer.setOnClickCallback((square) => picker.onSquareClicked(square));

      console.log(`✅ Initializing engine ${name()}`);
      engine = new WasmEngine();

      callback();
    })
    .catch(err => {
      console.error("❌ One or more images failed to load:", err);
    });
}

export function startNewGame(white: Player, black: Player, fen?: string) {
  board = new ChessBoard(fen || DEFAULT_FEN);
  gameController = new GameController(white, black);
  controller = gameController;

  controller.start()
}

// ---------------------------- Chess Board Wrapper -----------------------------
export class ChessBoard {
  private _gameState: WasmGame;
  private initialPos: string;

  private _boardString = '';
  private _legalMoves: string[] = [];
  private _legalMovesMap = new Map<string, Set<string>>();

  private history: WasmMove[];

  constructor(fen: string) {
    this._gameState = new WasmGame(fen);

    this.initialPos = `fen ${fen}`;
    this.history = [];

    this.updateInternal();
  }

  get gameState() {
    return this._gameState;
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
    const uci = `${initialPos} ${moves}`;
    return uci;
  }

  public lastMove(): WasmMove | null {
    return this.history.length > 0 ? this.history[this.history.length - 1] : null;
  }

  private updateInternal() {
    // board string
    this._boardString = this.gameState.board_string();
    // legal moves
    this._legalMoves = this.gameState.legal_moves();
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

  turn() {
    return this.gameState.turn();
  }

  makeMove(move_str: string) {
    const move = this.gameState.make_move(move_str);
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

class Picker {
  selected: string | null = null;

  public onSquareClicked(square: string) {
    console.log(`Square clicked: ${square}`);
    if (board!.legalMovesMap.has(square)) {
      this.selected = square;
    }
  }

  // private onMouseMove = (event: MouseEvent | TouchEvent) => {
  //   if (!this.resolveMove || this.selectingPromotion) return;
  //   const { x, y } = this.normalizeMouseEvent(event);
  //   this.x = x;
  //   this.y = y;
  //   renderer!.draw();
  // };

  // private onMouseUp = async (event: MouseEvent | TouchEvent) => {
  //   if (!this.resolveMove || !this.selected) return;
  //   const { x, y } = this.normalizeMouseEvent(event);
  //   const square = this.viewport.screenToSquare(x, y);

  //   if (board!.legalMovesMap.get(this.selected)?.has(square)) {
  //     let move = `${this.selected}${square}`;

  //     const piece = board!.getPieceAt(this.selected);

  //     let promotion = null;
  //     if (piece === 'P' && square[1] === '8') {
  //       promotion = true;
  //     } else if (piece === 'p' && square[1] === '1') {
  //       promotion = false;
  //     }

  //     if (promotion !== null) {
  //       this.selected = null;
  //       this.selectingPromotion = true;
  //       const promotionPiece = await this.waitForPromotionSelection(promotion, event);
  //       this.selectingPromotion = false;
  //       move += promotionPiece;
  //     }

  //     const resolve = this.resolveMove;
  //     this.resolveMove = null;
  //     resolve(move);
  //   }

  //   this.selected = null;
  //   renderer!.draw();
  // };

  // private waitForPromotionSelection(isWhite: boolean, event: MouseEvent | TouchEvent): Promise<string> {
  //   const { x, y } = this.normalizeMouseEvent(event);

  //   return new Promise((resolve) => {
  //     const container = document.createElement('div');
  //     container.id = 'promotion-dialog';
  //     container.style.position = 'absolute';
  //     container.style.left = `${x}px`;
  //     container.style.top = `${y}px`;
  //     container.style.display = 'flex';
  //     container.style.gap = '8px';
  //     container.style.zIndex = '9999';
  //     container.style.background = 'rgba(255, 255, 255, 0.95)';
  //     container.style.padding = '6px';
  //     container.style.border = '1px solid #ccc';
  //     container.style.borderRadius = '4px';
  //     container.style.boxShadow = '0 2px 6px rgba(0,0,0,0.3)';

  //     const pieces = ['Q', 'R', 'B', 'N'];

  //     for (let piece of pieces) {
  //       if (!isWhite) {
  //         piece = piece.toLowerCase();
  //       }

  //       const option = document.createElement('div');
  //       option.style.width = '64px';
  //       option.style.height = '64px';
  //       option.style.cursor = 'pointer';
  //       option.style.backgroundImage = `url(${PIECE_RES.get(piece)?.src})`;
  //       option.style.backgroundSize = 'contain';
  //       option.style.backgroundRepeat = 'no-repeat';
  //       option.style.backgroundPosition = 'center';

  //       option.onclick = () => {
  //         document.body.removeChild(container);
  //         resolve(piece);
  //       };

  //       container.appendChild(option);
  //     }

  //     document.body.appendChild(container);
  //   });
  // }
};

const picker = new Picker();

// ---------------------------- Game Controller -------------------------------
type GameState = 'waitingInput' | 'paused' | 'gameOver';

class GameController {
  private players: Player[];
  private gameState: GameState = 'waitingInput';
  private result = 'playing';

  constructor(white: Player, black: Player) {
    this.players = [white, black];
  }

  public start() {
    this.gameState = 'waitingInput';
    this.loop();
  }

  private loop() {
    if (board && renderer) {
      renderer.draw(board);
    }

    // return one frame later so the end result is rendered
    if (this.result !== 'playing') {
      alert(this.result);
      return;
    }

    switch (this.gameState) {
      case 'waitingInput': {
        const player = this.players[board!.turn()];
        const moveStr = player.tryGetMove(board!.uciPosition());
        if (moveStr) {
          const move = board!.makeMove(moveStr);
          if (move) {
            // @TODO: change to animation state
            console.log(board!.gameState.debug_string());
            this.result = board!.gameState.get_result();
          }
        }
      } break;
      case 'paused': {
        // Do nothing, wait for input to resume
      } break;
      default: throw new Error(`Unknown game state: ${this.gameState}`);
    }

    requestAnimationFrame(() => this.loop());
  }
}

export interface Player {
  tryGetMove(history: string): string | null;
}

export class BotPlayer implements Player {
  tryGetMove(history: string): string | null {
    const SEARCH_TIME = 2000;

    engine?.set_position(history);

    const bestMove = engine?.best_move(SEARCH_TIME);
    return bestMove || null;
  }
}

export class UIPlayer implements Player {
  private bufferedMove: string | null = null;

  provideMove(move: string) {
    this.bufferedMove = move;
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  tryGetMove(history: string): string | null {
    return this.bufferedMove;
  }
}

// ---------------------------- Utils ------------------------------------
function squareStringToIndex(square: string): number {
  const file = square.charCodeAt(0) - 97; // 'a' is 97
  const rank = parseInt(square[1], 10) - 1; // '1' is 1
  return rank * BOARD_SIZE + file;
}
