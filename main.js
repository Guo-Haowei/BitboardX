import init, { Engine } from './engine/pkg/engine.js';

async function run() {
  await init();
  const div = document.getElementById('result');

  const board = new Engine();
  div.textContent = board.to_string();
}

run();
