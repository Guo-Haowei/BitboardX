import * as renderer from './renderer.js'
import init, { Engine } from '../engine/pkg/engine.js';

let engine = null;

function updateBoard(fen) {
  const div = document.getElementById('result');
  try {
    engine.parse_fen(fen);
    div.textContent = engine.pretty_string();

    renderer.draw(engine.to_string());
  } catch (e) {
    console.error(`Error parsing '${fen}': ${e}`);
  }
}

async function run() {
  await init();

  engine = new Engine();
  renderer.init();

  document.getElementById('fenButton').addEventListener('click', () => {
    const fen = document.getElementById('fenInput').value;
    updateBoard(fen);
  });

  const fen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
  updateBoard(fen);
}

run();
