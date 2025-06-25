import { BOARD_SIZE, COLORS, PIECE_SYMBOLS } from './constants';
import { isLowerCase, fileRankToSquare } from './utils';
import { RuntimeModule, runtime } from './runtime';
import { picker } from './picker';
import { Listener, EVENT_MAP, Payload } from './message-queue';
import { WasmMove } from '../../bitboard_x/pkg/bitboard_x';
import { PIECE_RES } from './chess';

const GREEN_COLOR = 'rgba(0, 200, 0, 0.5)';
const RED_COLOR = 'rgba(200, 0, 0, 0.5)';
const YELLOW_COLOR_1 = 'rgba(150, 150, 0, 0.5)';
const YELLOW_COLOR_2 = 'rgba(200, 200, 0, 0.5)';

function getTileSize() {
  return runtime.display.tileSize;
}

export class Renderer implements RuntimeModule, Listener {
  private ctx: CanvasRenderingContext2D | null;
  private images: Map<string, HTMLImageElement>;
  private lastMove: string | null;

  public constructor() {
    this.ctx = null;
    this.images = PIECE_RES;
    this.lastMove = null;
  }

  public init(): boolean {
    runtime.messageQueue.subscribe('move', this);

    this.ctx = runtime.display.canvas.getContext('2d');
    if (!this.ctx) {
      return false;
    }

    this.ctx.font = '40px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';

    return true;
  }

  public handleMessage(event: string, payload?: Payload) {
    switch (event) {
      case EVENT_MAP.MOVE: {
        this.lastMove = payload as string || null;
      } break;
      default: break;
    }
  }

  public tick() {
    const canvas = runtime.display.canvas;
    const { ctx } = this;
    if (ctx) {
      ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight);
      ctx.font = `${runtime.display.tileSize / 2}px Arial`
      this.drawBoard();
      this.drawPieces(runtime.gameManager.board.board);
    }
  }

  private fillSquare(col: number, row: number, color: string) {
    if (!this.ctx) {
      return;
    }
    const tileSize = getTileSize();
    this.ctx.fillStyle = color;
    this.ctx.fillRect(col * tileSize, row * tileSize, tileSize, tileSize);
  }

  private drawBoard() {
    const { ctx } = this;
    if (!ctx) {
      return;
    }

    const { moves, square } = picker;
    const tileSize = getTileSize();

    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        const color = ((row + col) % 2 === 0 ? COLORS.light : COLORS.dark);
        this.fillSquare(col, row, color);

        const sq = fileRankToSquare(col, 7 - row);
        if (sq === square) {
          this.fillSquare(col, row, GREEN_COLOR);
        } else if (moves && moves.has(sq)) {
          this.fillSquare(col, row, RED_COLOR);
        }
        if (this.lastMove) {
          const src = this.lastMove.src_sq();
          const dst = this.lastMove.dst_sq();
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
    if (!this.ctx) {
      return;
    }

    const tileSize = getTileSize();
    const img = this.images.get(piece);
    if (img) {
      const half = tileSize / 2;
      this.ctx.drawImage(img, x - half, y - half, tileSize, tileSize);
    } else {
      this.ctx.fillStyle = isLowerCase(piece) ? 'black' : 'white';
      this.ctx.fillText(PIECE_SYMBOLS[piece], x, y);
      // console.log(`Drawing piece ${piece} at (${x}, ${y})`);
    }
  }

  private drawPieces(board: string) {
    if (!this.ctx) {
      return;
    }

    const animated = new Set<number>();
    const tileSize = getTileSize();

    const { animations } = runtime.animationManager;
    for (const animation of animations) {
      const { piece, dstFile, dstRank } = animation;
      const idx = dstFile + dstRank * BOARD_SIZE;
      animated.add(idx);
      const x = animation.x * tileSize + tileSize / 2;
      const y = animation.y * tileSize + tileSize / 2;
      this.drawPiece(piece, x, y);
    }

    for (let row = 0; row < BOARD_SIZE; ++row) {
      for (let col = 0; col < BOARD_SIZE; ++col) {
        const idx = (7 - row) * BOARD_SIZE + col;
        const piece = board[idx];
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
