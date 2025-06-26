/* eslint-disable @typescript-eslint/no-non-null-assertion */
import * as Chess from './chess';
import { writeMatchHTML } from './ui-generation';

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

class StreamingPlayer implements Chess.Player {
  private moveQueue: string[] = [];
  private waitingResolve: ((move: string) => void) | null = null;

  queueMove(bestmove: string): void {
    if (this.waitingResolve) {
      this.waitingResolve(bestmove);
      this.waitingResolve = null;
    } else {
      this.moveQueue.push(bestmove);
    }
  }

  async getMove(): Promise<string> {
    if (this.moveQueue.length > 0) {
      return this.moveQueue.shift()!;
    }
    return new Promise((resolve) => {
      this.waitingResolve = resolve;
    });
  };
}

const player = new StreamingPlayer();

class ChessSocket {
  private socket: WebSocket;

  constructor(url: string) {
    this.socket = new WebSocket(url);

    this.socket.onopen = () => {
      console.log("[WebSocket] Connected");
    };

    this.socket.onmessage = (event) => {
      const msg: string = event.data;
      let match = msg.match(/bestmove\s+(\w+)/);
      if (match) {
        player.queueMove(match[1]);
        return;
      }

      match = msg.match(/Started game (\d+) of (\d+) \((.+?) vs (.+?)\)/);
      if (match) {
        const whitePlayerDiv = document.getElementById('white-player') as HTMLDivElement;
        whitePlayerDiv.textContent = match[3];
        const blackPlayerDiv = document.getElementById('black-player') as HTMLDivElement;
        blackPlayerDiv.textContent = match[4];
        const game = Chess.createGame(player, player);
        game.start();
        return;
      }

      match = msg.match(/Finished game/);
      if (match) {
        // @TODO: restart
        return;
      }

      console.log("[WebSocket] Message received:", msg);
    };
  }

  send(msg: string) {
    this.socket.send(msg);
  }
}

const socket = new ChessSocket('ws://localhost:3000/ws');

async function main() {
  const canvas = document.getElementById('chessCanvas') as HTMLCanvasElement;
  canvas.tabIndex = 0;

  await Chess.initialize({ canvas, createUIPlayer: false }, async () => {
    document.getElementById('match-button')?.addEventListener('click', async () => {
      const whitePlayer = (document.getElementById('select-white') as HTMLDivElement).textContent;
      const blackPlayer = (document.getElementById('select-black') as HTMLDivElement).textContent;

      if (!whitePlayer || !blackPlayer) {
        alert('Please select both players before starting the match.');
        return;
      }

      socket.send(`match:${whitePlayer}:${blackPlayer}`);
    });
  });
}

main();