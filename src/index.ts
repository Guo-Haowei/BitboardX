import { BOARD_SIZE, CANVAS_ID, DEFAULT_FEN, TILE_SIZE } from './constants.js';
import { Game } from './game.js';
import { renderer } from './renderer.js';
import { inputManager } from './input.js';
import init from '../engine/pkg/engine.js';

let game: Game = new Game();
let canvas: HTMLCanvasElement | null = null;

// @TODO: get rid of this function
function updateBoard(fen: string) {
  game = new Game();
  if (!game.init(fen)) {
    return;
  }
}

function processEvents() {
  const { eventQueue } = inputManager;
  let event = null;
  while ((event = eventQueue.shift()) !== undefined) {
    switch (event.type) {
    case 'mousedown': game.onMouseDown(event.payload!); break;
    case 'mousemove': game.onMouseMove(event.payload!); break;
    case 'mouseup': game.onMouseUp(event.payload!); break;
    case 'undo': game.undo(); break;
    case 'redo': game.redo(); break;
    default: break;
    }
  }
}

function render() {
  renderer.draw(game.board);
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
  canvas.tabIndex = 0;
  canvas.style = 'position: absolute; top: 0px; left: 0px; right: 0px; bottom: 0px; margin: auto;';
  document.body.appendChild(canvas);
  return canvas;
}

async function run() {
  await init();

  initCanvas();
  renderer.init(canvas!);

  inputManager.init(canvas!);

  updateBoard(DEFAULT_FEN);

  gameLoop();
}

run();
