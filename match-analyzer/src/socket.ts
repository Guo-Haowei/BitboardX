/* eslint-disable @typescript-eslint/no-non-null-assertion */
import * as Chess from './chess';

type ServerMessage =
  | { type: "bestmove"; bestmove: string }
  | { type: "newgame"; game: number; total: number; white: string; black: string }

export class ChessSocket {
  private socket: WebSocket;
  private listeners = new Map<string, ((msg: ServerMessage) => void)[]>();

  constructor(url: string) {
    this.socket = new WebSocket(url);

    this.socket.onopen = () => {
      console.log("[WebSocket] Connected");
    };

    this.socket.onmessage = (event) => {
      const msg: ServerMessage = JSON.parse(event.data);
      if (msg.type === "newgame") {
        console.log(msg);
      } else {
        this.dispatch(msg);
      }
    };
  }

  private dispatch(msg: ServerMessage) {
    const listeners = this.listeners.get(msg.type);
    if (listeners) {
      for (const fn of listeners) fn(msg);
    }
  }

  on<T extends ServerMessage["type"]>(
    type: T,
    handler: (msg: Extract<ServerMessage, { type: T }>) => void
  ) {
    const arr = this.listeners.get(type) || [];
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    arr.push(handler as any);
    this.listeners.set(type, arr);
  }

  send(msg: string) {
    this.socket.send(msg);
  }
}

export class StreamingPlayer implements Chess.Player {
  private socket: ChessSocket;
  private moveQueue: string[] = [];
  private waitingResolve: ((move: string) => void) | null = null;

  constructor(socket: ChessSocket) {
    this.socket = socket;
    this.socket.on('bestmove', ({ bestmove }) => {
      if (this.waitingResolve) {
        this.waitingResolve(bestmove);
        this.waitingResolve = null;
      } else {
        this.moveQueue.push(bestmove);
      }
    });
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