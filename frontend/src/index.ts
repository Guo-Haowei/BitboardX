import { runtime } from './runtime';
import init from '../../pkg/bitboard_x';

function tick() {
  runtime.tick();
  requestAnimationFrame(tick);
}

async function run() {
  await init();

  if (runtime.init()) {
    tick();
  }
};

run();
