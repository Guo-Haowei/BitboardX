import { RuntimeModule, runtime } from "./runtime";
import { picker } from './picker';

class EventMap {
  public readonly NEW_GAME = 'newgame';
  public readonly GAME_OVER = 'gameover';
  public readonly REQUEST_PLAYER_INPUT = 'request-player-input';
  public readonly MOVE = 'move';
  public readonly ANIMATION_DONE = 'animation-done';
};

export const Message = new EventMap();

const DEBUG = false;
// const DEBUG = true;

export interface Listener {
  handleMessage(message: string): void;
};

export class MessageQueue implements RuntimeModule {
  private queue: string[];
  private listeners: Map<string, Listener[]>;

  constructor() {
    this.queue = [];
    this.listeners = new Map<string, Listener[]>();
    for (const key of Object.keys(Message)) {
      const event = (Message as object)[key];
      this.listeners.set(event, []);
    }
  }

  public init(): boolean {
    const canvas = runtime.display.canvas;
    const getMousePosition = (canvas: HTMLCanvasElement, e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      return { x, y };
    };

    document.getElementById('fenButton')?.addEventListener('click', () => {
      this.emit(Message.NEW_GAME);
    });

    canvas.addEventListener('mousedown', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      console.log(`Mouse down at (${x}, ${y})`);
      picker.onMouseUp(x, y);
    });

    window.addEventListener('resize', () => {
      runtime.display.onResize();
    });

    return true;
  }

  public tick(): void {
    this.flush();
  }

  public subscribe(event: string, listener: Listener): void {
    if (!this.listeners.has(event)) {
      console.error(`event '${event}' is not supported.`);
      return;
    }
    const listeners = this.listeners.get(event);
    if (listeners) {
      listeners.push(listener);
    } else {
      console.error(`No listeners found for event '${event}'.`);
    }
  }

  public emit(message: string): void {
    if (DEBUG) {
      console.log(`>>> emitting message ${message}`);
    }
    this.queue.push(message);
  }

  public flush(): void {
    const count = this.queue.length;
    for (let i = 0; i < count; ++i) {
      const message = this.queue.shift();
      if (!message) {
        console.error('MessageQueue: flush called with empty queue');
        break;
      }
      const event = message.split(':')[0];
      const listeners = this.listeners.get(event);
      if (!listeners) {
        console.error(`MessageQueue: no listeners found for event '${event}'`);
        continue;
      }
      listeners.forEach((listener) => {
        if (DEBUG) {
          console.log(`<<< handling message: ${message} by listener: ${listener.constructor.name}`);
        }
        listener.handleMessage(message);
      });
    }
  }
};
