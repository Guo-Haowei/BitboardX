import { BOARD_SIZE, CANVAS_ID, DEFAULT_FEN, TILE_SIZE } from './constants.js';
import { Game } from './game.js';
import { renderer } from './renderer.js'
import { InputManager } from './input.js';
import init from '../engine/pkg/engine.js';

let game = null;
let canvas = null;
let inputManager = null;

// @TODO: get rid of this function
function updateBoard(fen) {
  game = new Game();
  if (!game.init(fen)) {
    return;
  }
}

function processEvents() {
  let { eventQueue } = inputManager;
  while (eventQueue.length > 0) {
    const event = eventQueue.shift();
    switch (event.type) {
      case 'mousedown': game.onMouseDown(event); break;
      case 'mousemove': game.onMouseMove(event); break
      case 'mouseup': game.onMouseUp(event); break;
      case 'undo': game.undo(); break;
      case 'redo': game.redo(); break;
      default: break;
    }
  }
}

function render() {
  const {boardString} = game;
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
  canvas.style = 'position: absolute; top: 0px; left: 0px; right: 0px; bottom: 0px; margin: auto;';
  document.body.appendChild(canvas);
  return canvas;
}

async function run() {
  await init();

  initCanvas();
  renderer.init(canvas);

  inputManager = new InputManager();
  inputManager.init(canvas);

  updateBoard(DEFAULT_FEN);

  gameLoop();
}

run();
