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

  const input = document.getElementById('fen-input') as HTMLInputElement;
  const fen = input.value;

  return Chess.createGame(
    whitePlayer,
    blackPlayer,
    fen,
  );
}

async function main() {
  const canvas = document.getElementById('chessCanvas') as HTMLCanvasElement;
  canvas.tabIndex = 0;
  canvas.style.margin = '20px auto';

  await Chess.initialize({ canvas, createUIPlayer: true }, async () => {
    let controller = createGame();
    const result = await controller.start();
    if (result !== 'playing') alert(result);

    document.getElementById('start-button')?.addEventListener('click', async () => {
      if (controller) {
        controller.stop();
      }

      controller = createGame();
      const result = await controller.start();
      if (result !== 'playing') alert(result);
    });
  });
}

main();