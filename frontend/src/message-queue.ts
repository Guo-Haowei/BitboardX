import { RuntimeModule, runtime } from "./runtime";
import { picker } from './picker';
import { WasmMove } from "../../bitboard_x/pkg/bitboard_x";

const DEBUG = false;
// const DEBUG = true;

class EventMap {
  public readonly NEW_GAME = 'newgame';
  public readonly GAME_OVER = 'gameover';
  public readonly REQUEST_INPUT = 'request-input';
  public readonly MOVE = 'move';
  public readonly ANIMATION_DONE = 'animation-done';
};

export type Payload = WasmMove;

export const EVENT_MAP = new EventMap();

interface Message {
  event: string;
  payload?: Payload;
};

export interface Listener {
  handleMessage(event: string, payload?: Payload): void;
};

export class MessageQueue implements RuntimeModule {
  private queue: Message[];
  private listeners: Map<string, Listener[]>;

  constructor() {
    this.queue = [];
    this.listeners = new Map<string, Listener[]>();
    for (const key of Object.keys(EVENT_MAP)) {
      const event = EVENT_MAP[key as keyof EventMap];
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
      this.emit({ event: EVENT_MAP.NEW_GAME });
    });

    canvas.addEventListener('mousedown', (e) => {
      const { x, y } = getMousePosition(canvas, e);
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

  public emit(msg: Message) {
    if (DEBUG) {
      console.log(`>>> emitting message ${msg.event} with payload: ${msg.payload}`);
    }
    this.queue.push(msg);
  }

  public flush(): void {
    const count = this.queue.length;
    for (let i = 0; i < count; ++i) {
      const message = this.queue.shift();
      if (!message) {
        console.error('MessageQueue: flush called with empty queue');
        break;
      }
      const { event, payload } = message;
      const listeners = this.listeners.get(event);
      if (!listeners) {
        console.error(`MessageQueue: no listeners found for event '${event}'`);
        continue;
      }
      listeners.forEach((listener) => {
        if (DEBUG) {
          console.log(`<<< handling message: ${message} by listener: ${listener.constructor.name}`);
        }
        listener.handleMessage(event, payload);
      });
    }
  }
};
