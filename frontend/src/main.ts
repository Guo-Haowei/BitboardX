import * as Chess from './chess';

async function run() {
  const controller = new Chess.GameController(
    new Chess.BotPlayer(), // White player
    new Chess.BotPlayer(), // Black player
  );

  await controller.start();
};

Chess.initialize(() => {
  run();
});
