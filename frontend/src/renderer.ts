import { BOARD_SIZE, COLORS, PIECE_SYMBOLS, TILE_SIZE } from './constants';
import { isLowerCase, fileRankToSquare } from './utils';
import { RuntimeModule, runtime } from './runtime';
import { picker } from './picker';

const GREEN_COLOR = 'rgba(0, 200, 0, 0.5)';
const RED_COLOR = 'rgba(200, 0, 0, 0.5)';

function loadPieceImages() {
  const pieces = new Map<string, HTMLImageElement>();
  const pieceCodes = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];

  pieceCodes.forEach(code => {
    const color = isLowerCase(code) ? 'b' : 'w';
    const id = `img-${color}${code.toUpperCase()}`;
    console.log(`Loading image with id: ${id}`);
    const img = document.getElementById(id) as HTMLImageElement;
    console.log(`Loading image for piece: ${img}`);
    pieces[code] = img;

  });

  return pieces;
}

export class Renderer implements RuntimeModule {
  private ctx: CanvasRenderingContext2D | null;
  private images: Map<string, HTMLImageElement>;

  public constructor() {
    this.ctx = null;
    // this.images = new Map<string, HTMLImageElement>();
    this.images = loadPieceImages();
  }

  public init(): boolean {
    this.ctx = runtime.display.canvas.getContext('2d');
    if (!this.ctx) {
      return false;
    }

    this.ctx.font = '40px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';

    return true;
  }

  public tick() {
    this.drawBoard();
    this.drawPieces(runtime.gameManager.board.board);
  }

  private fillSquare(col: number, row: number, color: string) {
    if (!this.ctx) {
      return;
    }
    this.ctx.fillStyle = color;
    this.ctx.fillRect(col * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE);
  }

  private drawBoard() {
    if (!this.ctx) {
      return;
    }

    const { moves, square } = picker;

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
      }
    }

    // draw file labels
    this.ctx.fillStyle = 'black';
    for (let file = 0; file < BOARD_SIZE; ++file) {
      const x = file * TILE_SIZE + TILE_SIZE / 2;
      const y = BOARD_SIZE * TILE_SIZE + TILE_SIZE / 2;
      this.ctx.fillText(String.fromCharCode(97 + file).toString(), x, y);
    }

    // draw rank labels
    for (let row = 0; row < BOARD_SIZE; row++) {
      const x = BOARD_SIZE * TILE_SIZE + TILE_SIZE / 2;
      const y = row * TILE_SIZE + TILE_SIZE / 2;
      this.ctx.fillText((8 - row).toString(), x, y);
    }
  }

  private drawPiece(piece: string, x: number, y: number) {
    if (!this.ctx) {
      return;
    }

    const img = this.images[piece];
    if (img) {
      const half = TILE_SIZE / 2;
      this.ctx.drawImage(img, x - half, y - half, TILE_SIZE, TILE_SIZE);
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

    const { animations } = runtime.animationManager;
    for (const animation of animations) {
      const { piece, dstFile, dstRank } = animation;
      const idx = dstFile + dstRank * BOARD_SIZE;
      animated.add(idx);
      const x = animation.x * TILE_SIZE + TILE_SIZE / 2;
      const y = animation.y * TILE_SIZE + TILE_SIZE / 2;
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
        const x = col * TILE_SIZE + TILE_SIZE / 2;
        const y = row * TILE_SIZE + TILE_SIZE / 2;
        this.drawPiece(piece, x, y);
      }
    }
  }
}
