import { BOARD_SIZE, COLORS, PIECE_SYMBOLS, TILE_SIZE } from './constants';
import { isLowerCase } from './utils';
import { RuntimeModule, runtime } from './runtime';

export class Renderer implements RuntimeModule {
  private ctx: CanvasRenderingContext2D | null;

  public constructor() {
    this.ctx = null;
  }

  public getName(): string {
    return 'Renderer';
  }

  public init(): boolean {
    this.ctx = runtime.display.canvas.getContext('2d')!;
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
    this.drawPieces(runtime.game.board);
  }

  private drawBoard() {
    if (!this.ctx) {
      return;
    }

    const { selectedPiece } = runtime.game;
    const moves = selectedPiece ? selectedPiece.moves : 0n;

    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        const color = ((row + col) % 2 === 0 ? COLORS.light : COLORS.dark);
        this.ctx.fillStyle = color;
        this.ctx.fillRect(col * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE);

        if (moves & BigInt(1n << BigInt(col + (7 - row) * BOARD_SIZE))) {
          this.ctx.fillStyle = 'rgba(255, 0, 0, 0.5)';
          this.ctx.fillRect(col * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE);
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

  private drawPieces(board: string) {
    if (!this.ctx) {
      return;
    }

    for (let row = 0; row < BOARD_SIZE; ++row) {
      for (let col = 0; col < BOARD_SIZE; ++col) {
        const c = board[row * BOARD_SIZE + col];
        if (c === '.') {
          continue;
        }
        const piece = PIECE_SYMBOLS[c];
        const x = col * TILE_SIZE + TILE_SIZE / 2;
        const y = row * TILE_SIZE + TILE_SIZE / 2;
        this.ctx.fillStyle = isLowerCase(c) ? 'black' : 'white';
        this.ctx.fillText(piece, x, y);
      }
    }

    const { selectedPiece } = runtime.game;
    if (selectedPiece) {
      const { piece, x, y } = selectedPiece;
      this.ctx.fillStyle = isLowerCase(piece) ? 'black' : 'white';
      this.ctx.fillText(PIECE_SYMBOLS[piece], x, y);
    }
  }
}
