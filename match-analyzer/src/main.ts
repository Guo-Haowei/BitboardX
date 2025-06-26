import * as Chess from './chess';
import { ChessSocket, StreamingPlayer } from './socket';

interface MatchData {
  player1: string;
  player2: string;
  wins: number;
  losses: number;
  draws: number;
}

function writeMatchHTML(matchData: MatchData): string {
  const { player1, player2, wins, losses, draws } = matchData;
  const totalGames = (wins + losses + draws) / 100; // Assuming the data is in percentage format;
  return `
  <div class="bg-[#2b2b2b] p-4 rounded-lg">
    <div class="flex justify-between items-center mb-2">
      <h2 class="text-lg font-semibold">${player1} vs ${player2}</h2>
        <span class="text-sm text-gray-400">100 games</span>
    </div>
    <div class="text-sm text-gray-300 mb-2">
      Wins: <span class="text-green-400">${wins}</span>
      Draws: <span class="text-gray-400">${draws}</span>
      Losses: <span class="text-red-400">${losses}</span>
    </div>
    <div class="flex h-4 w-full overflow-hidden rounded bg-gray-700 mb-2">
      <div class="bg-green-500" style="width: ${wins / totalGames}%"></div>
      <div class="bg-gray-500" style="width: ${draws / totalGames}%"></div>
      <div class="bg-red-500" style="width: ${losses / totalGames}%"></div>
    </div>
    <details class="text-base">
      <summary class="cursor-pointer text-gray-300 hover:text-white">View Matches</summary>
      <div class="mt-2 space-y-2 max-h-40 overflow-y-auto text-sm bg-[#1a1a1a] p-2 rounded">
        <div><strong>Round 1 PGN:</strong> 1.e4 e5 2.Nf3 Nc6 3.Bb5 a6</div>
        <div><strong>Round 2 PGN:</strong> 1.d4 d5 2.c4 e6 3.Nc3 Nf6</div>
        <div><strong>Round 3 PGN:</strong> 1.e4 c5 2.Nf3 d6 3.d4 cxd4</div>
      </div>
    </details>
  </div>`;
}

async function loadMeta() {
  [{
    player1: 'Player A',
    player2: 'Player B',
    wins: 60,
    losses: 30,
    draws: 10
  }, {
    player1: 'Player C',
    player2: 'Player D',
    wins: 50,
    losses: 40,
    draws: 10
  }].forEach(match => {
    const matchHTML = writeMatchHTML(match);
    const matchPanel = document.getElementById('match-panel') as HTMLDivElement;
    matchPanel.innerHTML += matchHTML;
  });
  // try {
  //   const response = await fetch('http://localhost:3000/meta');
  //   if (!response.ok) throw new Error(`HTTP error ${response.status}`);
  //   console.log(`Response status: ${response.status}`);

  //   const data = await response.json(); // assuming it's an array of { player1, player2, result, file }

  //   let matchesHTML = '';
  //   data.forEach((match: MatchData) => {
  //     matchesHTML += writeMatchHTML(match);
  //   });
  //   const matchPanel = document.getElementById('match-panel') as HTMLDivElement;
  //   matchPanel.innerHTML = matchesHTML;
  // } catch (err) {
  //   console.error('Failed to load meta:', err);
  // }
}

window.onload = loadMeta;

function initSelectEngineButton(color: string) {
  const button = document.getElementById(`select-${color}`) as HTMLDivElement;
  const fileInput = document.getElementById(`${color}-player-input`) as HTMLInputElement;
  // const displayName = document.getElementById(`${color}-player`) as HTMLSpanElement;

  button.addEventListener('click', () => {
    fileInput.click();
  });

  fileInput.addEventListener('change', (e) => {
    const input = e.target as HTMLInputElement;
    if (!input.files) return;

    const file = Array.from(input.files)[0];
    const name = file.name.split('.').slice(0, -1).join('.');
    button.textContent = name;
  });
}

initSelectEngineButton('white');
initSelectEngineButton('black');

const socket = new ChessSocket('ws://localhost:3000/ws');

async function main() {
  const canvas = document.getElementById('chessCanvas') as HTMLCanvasElement;
  canvas.tabIndex = 0;

  await Chess.initialize({ canvas }, async () => {
    document.getElementById('match-button')?.addEventListener('click', async () => {
      const whitePlayer = (document.getElementById('select-white') as HTMLDivElement).textContent;
      const blackPlayer = (document.getElementById('select-black') as HTMLDivElement).textContent;

      console.log('Starting match with players:', whitePlayer, blackPlayer);

      if (!whitePlayer || !blackPlayer) {
        alert('Please select both players before starting the match.');
        return;
      }

      socket.send(`match:${whitePlayer}:${blackPlayer}`);

      const player1 = new StreamingPlayer(socket);
      const player2 = new StreamingPlayer(socket);
      while (true) {
        const game = Chess.createGame(player1, player2);
        await game.start();
      }
    });
  });
}

main();