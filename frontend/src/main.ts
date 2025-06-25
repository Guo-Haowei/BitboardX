import * as Chess from './chess';

let controller: Chess.GameController | null = null;

function createGame() {
  const selectPlayer = (player: string) => {
    const element = document.querySelector(`input[name="${player}"]:checked`);
    if (element && element instanceof HTMLInputElement) {
      console.log(element);
      if (element.value === 'bot') return new Chess.BotPlayer();
      if (element.value === 'human') return new Chess.BotPlayer();
    }

    throw new Error(`Invalid player: ${player}`);
  };

  const whitePlayer = selectPlayer('white-player');
  const blackPlayer = selectPlayer('black-player');

  const controller = new Chess.GameController(
    whitePlayer,
    blackPlayer,
    document.getElementById('fen-input')?.textContent || undefined
  );

  return controller;
}

Chess.initialize(() => {
  document.getElementById('start-button')?.addEventListener('click', () => {
    if (controller) {
      controller.stop(); // gracefully cancel current game
    }

    controller = createGame();
    controller.start();
  });

  controller = createGame();
  controller.start();
});
