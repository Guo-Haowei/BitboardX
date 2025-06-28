import * as Chess from './chess';

type GameInfo = [Chess.Player, Chess.Player, string];

function collectGameInfo(): GameInfo {
  const selectPlayer = (player: string) => {
    const element = document.querySelector(`input[name="${player}"]:checked`);
    if (element && element instanceof HTMLInputElement) {
      if (element.value === 'bot') return new Chess.BotPlayer();
      if (element.value === 'human') return new Chess.UIPlayer();
    }

    throw new Error(`Invalid player: ${player}`);
  };

  const white = selectPlayer('white-player');
  const black = selectPlayer('black-player');

  const input = document.getElementById('fen-input') as HTMLInputElement;
  const fen = input.value;

  return [white, black, fen];
}

async function main() {
  const canvas = document.getElementById('chess-board') as HTMLCanvasElement;
  canvas.tabIndex = 0;

  const controller = await Chess.initialize({ canvas });

  document.getElementById('start-button')?.addEventListener('click', async () => {

    console.log('Starting new game...');

    controller.newGame(...collectGameInfo());
  });

  document.getElementById('resume-button')?.addEventListener('click', () => {
    controller.resume();
  });

  document.getElementById('undo-button')?.addEventListener('click', () => {
    controller.undo();
  });

  controller.newGame(...collectGameInfo());
}

main();
