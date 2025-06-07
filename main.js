import init, { add } from './engine/pkg/engine.js';

async function run() {
  await init();
  const a = 6;
  const b = 7;
  const result = add(a, b);

  const resultDiv = document.getElementById('result');
  resultDiv.textContent = `${a} + ${b} = ${result}`;
}

run();
