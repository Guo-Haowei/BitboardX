import { runtime } from './runtime';
import init, { name } from '../../bitboard_x/pkg/bitboard_x';
import { initializeChess } from './chess';


async function run() {
  await init();

  console.log(`Running ${name()}`);

  if (runtime.init()) {
    await runtime.gameController?.start();
  }
};

initializeChess(() => {
  run();
});
