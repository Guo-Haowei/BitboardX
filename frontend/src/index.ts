import { runtime } from './runtime';
import init, { name } from '../../pkg/bitboard_x';

function tick() {
  runtime.tick();
  requestAnimationFrame(tick);
}

async function run() {
  await init();

  // eslint-disable-next-line no-console
  console.log(`Running ${name()}`);

  if (runtime.init()) {
    tick();
  }
};

run();
