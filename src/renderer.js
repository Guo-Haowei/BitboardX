import { BOARD_SIZE, COLORS, PIECE_SYMBOLS, TILE_SIZE } from './constants.js';
import { isLowerCase } from './stringUtils.js';

class Renderer {
  constructor() {
    this.ctx = null;
  }

  init() {
    const canvas = document.getElementById('chessboard');
    this.ctx = canvas.getContext('2d');
    this.ctx.font = '40px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';
  }

  drawBoard(selectedSqaure, moves) {
    selectedSqaure = selectedSqaure || -1;
    moves = moves || 0n;

    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        const square = col + row * BOARD_SIZE;

        let color = null;
        if (square === selectedSqaure) {
          color = COLORS.green;
        } else if (moves & BigInt(1n << BigInt(col + (7 - row) * BOARD_SIZE))) {
          color = COLORS.red;
        } else {
          color = ((row + col) % 2 === 0 ? COLORS.light : COLORS.dark);
        }
        this.ctx.fillStyle = color;
        this.ctx.fillRect(col * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE);
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

  drawPieces(boardString) {
    for (let row = 0; row < BOARD_SIZE; ++row) {
      for (let col = 0; col < BOARD_SIZE; ++col) {
        const c = boardString[row * BOARD_SIZE + col];
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
  }

  draw(context) {
    const { boardString, moves, selectedSqaure } = context;
    this.drawBoard(selectedSqaure, moves);
    this.drawPieces(boardString);
  }
}

export const renderer = new Renderer();
