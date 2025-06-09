import { BOARD_SIZE, CANVAS_ID, DEFAULT_FEN, PIECE_SYMBOLS, TILE_SIZE } from './constants.js';
import { Game } from './game.js';
import { renderer } from './renderer.js'
import init from '../engine/pkg/engine.js';

let game = null;
let canvas = null;

const eventQueue = [];

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

  prettyPrint(game.engine.to_string());
}

function setupListeners() {

  document.getElementById('fenButton').addEventListener('click', () => {
    const fen = document.getElementById('fenInput').value;
    updateBoard(fen);
  });

  const getMousePosition = (canvas, e) => {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    return {x, y};
  };

  canvas.addEventListener('mousedown', (e) => {
    const { x, y } = getMousePosition(canvas, e);
    eventQueue.push({ type: 'mousedown', x, y });
  });

  canvas.addEventListener('mousemove', (e) => {
    const { x, y } = getMousePosition(canvas, e);
    eventQueue.push({ type: 'mousemove', x, y});
  });

  canvas.addEventListener('mouseup', (e) => {
    const { x, y } = getMousePosition(canvas, e);
    eventQueue.push({ type: 'mouseup', x, y });
  });
}

function processEvents() {
  while (eventQueue.length > 0) {
    const event = eventQueue.shift();
    switch (event.type) {
      case 'mousedown': game.onMouseDown(event); break;
      case 'mousemove': game.onMouseMove(event); break
      case 'mouseup': game.onMouseUp(event); break;
      default: break;
    }
  }
}

function render() {
  const boardString = game.engine.to_string();
  renderer.draw({ boardString });
}

function gameLoop() {
  processEvents();
  render();
  requestAnimationFrame(gameLoop); // schedule next frame
}

function initCanvas() {
  canvas = document.createElement('canvas');
  canvas.id = CANVAS_ID;
  canvas.width = TILE_SIZE * (BOARD_SIZE + 1);
  canvas.height = TILE_SIZE * (BOARD_SIZE + 1);
  canvas.tabindex = 0;
  canvas.style = 'position: absolute; top: 0px; left: 0px; right: 0px; bottom: 0px; margin: auto; border:2px solid blue';
  document.body.appendChild(canvas);
  return canvas;
}

async function run() {
  await init();

  initCanvas();
  renderer.init(canvas);

  setupListeners();

  updateBoard(DEFAULT_FEN);

  gameLoop();
}

run();
