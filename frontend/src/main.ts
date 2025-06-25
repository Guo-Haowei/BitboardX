import * as Chess from './chess';

function createGame() {
  const selectPlayer = (player: string) => {
    const element = document.querySelector(`input[name="${player}"]:checked`);
    if (element && element instanceof HTMLInputElement) {
      if (element.value === 'bot') return new Chess.BotPlayer();
      if (element.value === 'human') return new Chess.UIPlayer();
    }

    throw new Error(`Invalid player: ${player}`);
  };

  const whitePlayer = selectPlayer('white-player');
  const blackPlayer = selectPlayer('black-player');

  return Chess.createGame(
    whitePlayer,
    blackPlayer,
    document.getElementById('fen-input')?.textContent || undefined
  );
}

Chess.initialize(() => {
  let controller = createGame();
  document.getElementById('start-button')?.addEventListener('click', () => {
    if (controller) {
      controller.stop(); // gracefully cancel current game
    }

    controller = createGame();
    controller.start();
  });

  controller.start();
});
