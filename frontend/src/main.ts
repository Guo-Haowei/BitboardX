import { runtime } from './runtime';
import init, { name } from '../../bitboard_x/pkg/bitboard_x';
import { initializeChess } from './chess';

function tick() {
  runtime.tick();

  requestAnimationFrame(tick);
}

async function run() {
  await init();

  console.log(`Running ${name()}`);

  if (runtime.init()) {
    tick();
  }
};

initializeChess(() => {
  run();
});
