import { ChessBoard, PIECE_RES } from './chess';

// @TODO: allow user to configure the colors
const GREEN_COLOR = 'rgba(0, 200, 0, 0.5)';
const RED_COLOR = 'rgba(200, 0, 0, 0.5)';
const YELLOW_COLOR = 'rgba(200, 200, 0, 0.5)';
const LIGHT_SQUARE_COLOR = 'rgba(240, 217, 181, 1)';
const DARK_SQUARE_COLOR = 'rgba(181, 136, 99, 1)';

export class Renderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private squareSize = 0;
  private onclickCallback?: ((square: string) => void);

  public constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    canvas.addEventListener('resize', () => {
      this.onResize();
    });
    canvas.addEventListener('click', (e) => {
      this.onclickCallback?.(this.screenToSquare(e.offsetX, e.offsetY));
    });
    this.onResize();

    this.ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
    if (!this.ctx) {
      throw new Error("Failed to get canvas context");
    }
  }

  setOnClickCallback(callback: (square: string) => void) {
    this.onclickCallback = callback;
  }

  draw(board: ChessBoard, selected?: string) {
    const { ctx, canvas } = this;
    const { width, height } = canvas;
    if (board) {
      ctx.clearRect(0, 0, width, height);
      this.drawBoard(board, selected);
      this.drawPieces(board);
    }
  }

  private fillSquare(col: number, row: number, color: string) {
    const { squareSize, ctx } = this;
    ctx.fillStyle = color;
    ctx.fillRect(col * squareSize, row * squareSize, squareSize, squareSize);
  }

  private drawBoard(board: ChessBoard, selected?: string) {
    const { ctx, squareSize } = this;

    selected = selected || '';
    const legalMoves = board.legalMovesMap.get(selected);

    const lastMove = board.lastMove();
    for (let idx = 0; idx < 64; ++idx) {
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
    for (let i = 0; i < 8; ++i) {
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

    // @TODO: fix this
    const { selected, x, y } = { selected: null, x: -1, y: -1 };
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

  private onResize() {
    const { clientWidth, clientHeight } = this.canvas;
    const minSize = 200;
    const size = Math.max(minSize, Math.min(clientWidth, clientHeight));
    this.canvas.width = size;
    this.canvas.height = size;
    this.squareSize = size / 8;
  }

  private screenToSquare(x: number, y: number): string {
    const file = Math.floor(x / this.squareSize);
    const rank = 7 - Math.floor(y / this.squareSize);
    return fileRankToSquare(file, rank);
  }
}

function squareToFileRank(square: number): [number, number] {
  return [square % 8, Math.floor(square / 8)];
}

function fileRankToSquare(file: number, rank: number) {
  return `${String.fromCharCode(97 + file)}${rank + 1}`;
}
