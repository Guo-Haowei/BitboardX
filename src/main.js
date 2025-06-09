import { BOARD_SIZE, DEFAULT_FEN, PIECE_SYMBOLS, TILE_SIZE } from './constants.js';
import { Game } from './game.js';
import { renderer } from './renderer.js'
import init from '../engine/pkg/engine.js';

let game = null;

function prettyPrint(boardString) {
  let result = '';
  for (let y = 0; y < BOARD_SIZE; ++y) {
    let line = `${8 - y} `;
    for (let x = 0; x < BOARD_SIZE; ++x) {
      const intex = x + y * BOARD_SIZE;
      const c = boardString[intex];
      if (c === '.') {
        line += '・ ';
        continue;
      }
      line += PIECE_SYMBOLS[c] + ' ';
    }
    result += `${line}\n`;
  }
  result += "  ａ ｂ ｃ ｄ ｅ ｆ ｇ ｈ\n\n";
  console.log(result);
}

function updateBoard(fen) {
  game = new Game();
  if (!game.init(fen)) {
    return;
  }

  const { engine } = game;

  const boardString = engine.to_string();

  renderer.draw({ boardString });
  prettyPrint(boardString);
}

function setupListeners() {
  const canvas = document.getElementById('chessboard');

  canvas.addEventListener('click', (event) => {
    const rect = canvas.getBoundingClientRect();
    const x = Math.floor((event.clientX - rect.left) / TILE_SIZE);
    const y = Math.floor((event.clientY - rect.top) / TILE_SIZE);

    if (x < BOARD_SIZE && y < BOARD_SIZE) {
      game.onClick(y, x);
    }
  });

  document.getElementById('fenButton').addEventListener('click', () => {
    const fen = document.getElementById('fenInput').value;
    updateBoard(fen);
  });
}

async function run() {
  await init();

  renderer.init();

  setupListeners();

  updateBoard(DEFAULT_FEN);
}

run();
