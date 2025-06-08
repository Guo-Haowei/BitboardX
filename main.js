import init, { Engine } from './engine/pkg/engine.js';

async function run() {
  await init();

  document.getElementById('fenButton').addEventListener('click', () => {
    const fen = document.getElementById('fenInput').value;
    updateBoard(fen);
  });

  const fen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
  updateBoard(fen);
}

function updateBoard(fen) {
  const div = document.getElementById('result');
  const engine = new Engine();
  const err = engine.parse_fen(fen);
  div.textContent = engine.to_string();
}

run();
