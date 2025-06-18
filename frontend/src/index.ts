import { runtime } from './runtime';
import init, { name } from '../../pkg/bitboard_x';
import { messageQueue } from './message-queue';

function tick() {
  runtime.tick();

  messageQueue.flush();

  requestAnimationFrame(tick);
}

async function run() {
  await init();


  console.log(`Running ${name()}`);

  if (runtime.init()) {
    tick();
  }
};

run();
