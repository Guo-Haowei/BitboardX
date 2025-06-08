import { BOARD_SIZE, DEFAULT_FEN, TILE_SIZE } from './constants.js';
import { Game } from './game.js';
import { renderer } from './renderer.js'
import init from '../engine/pkg/engine.js';

let game = null;

function updateBoard(fen) {
  game = new Game();
  if (!game.init(fen)) {
    return;
  }

  const { engine } = game;

  const div = document.getElementById('result');
  div.textContent = engine.pretty_string();

  const boardString = engine.to_string();
  renderer.draw({ boardString });
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
