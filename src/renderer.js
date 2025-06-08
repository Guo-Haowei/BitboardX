import { BOARD_SIZE, COLORS, PIECE_SYMBOLS, TILE_SIZE } from './constants.js';
import { isLowerCase } from './stringUtils.js';

let canvas = null;
let ctx = null;

export function init() {
  canvas = document.getElementById('chessboard');
  ctx = canvas.getContext('2d');

  ctx.font = '40px Arial';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
}

function drawBoard() {
  for (let row = 0; row < BOARD_SIZE; row++) {
    for (let col = 0; col < BOARD_SIZE; col++) {
      const isLight = (row + col) % 2 === 0;
      ctx.fillStyle = isLight ? COLORS.light : COLORS.dark;
      ctx.fillRect(col * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE);
    }
  }

  // draw file labels
  ctx.fillStyle = 'black';
  for (let file = 0; file < BOARD_SIZE; ++file) {
    const x = file * TILE_SIZE + TILE_SIZE / 2;
    const y = BOARD_SIZE * TILE_SIZE + TILE_SIZE / 2;
    ctx.fillText(String.fromCharCode(97 + file).toString(), x, y);
  }

  // draw rank labels
  for (let row = 0; row < BOARD_SIZE; row++) {
    const x = BOARD_SIZE * TILE_SIZE + TILE_SIZE / 2;
    const y = row * TILE_SIZE + TILE_SIZE / 2;
    ctx.fillText((8 - row).toString(), x, y);
  }
}

function drawPieces(boardString) {
  for (let row = 0; row < BOARD_SIZE; ++row) {
    for (let col = 0; col < BOARD_SIZE; ++col) {
      const c = boardString[row * BOARD_SIZE + col];
      if (c === '.') {
        continue;
      }
      const piece = PIECE_SYMBOLS[c];
      const x = col * TILE_SIZE + TILE_SIZE / 2;
      const y = row * TILE_SIZE + TILE_SIZE / 2;
      ctx.fillStyle = isLowerCase(c) ? 'black' : 'white';
      ctx.fillText(piece, x, y);
    }
  }
}

export function draw(boardString) {
  drawBoard();
  drawPieces(boardString);
}
