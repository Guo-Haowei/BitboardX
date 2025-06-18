import { Point2D } from './utils';
import { RuntimeModule, runtime } from './runtime';

export type Event = {
  type: string;
  payload: Point2D | string | null;
};

export interface EventListener {
  // eslint-disable-next-line no-unused-vars
  handleEvent(event: Event): void;
}

export class EventManager implements RuntimeModule {
  private queue: Event[];
  private listeners: EventListener[];

  public constructor() {
    this.queue = [];
    this.listeners = [];
  }

  public getName(): string {
    return 'EventManager';
  }

  public addListener(listener: EventListener): void {
    this.listeners.push(listener);
  }

  public removeListener(listener: EventListener): void {
    this.listeners = this.listeners.filter(l => l !== listener);
  }

  public init(): boolean {
    const canvas = runtime.display.canvas;
    const getMousePosition = (canvas: HTMLCanvasElement, e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      return {x, y};
    };

    const { queue } = this;

    document.getElementById('undoButton')?.addEventListener('click', () => {
      queue.push({ type: 'undo', payload: null });
    });

    document.getElementById('redoButton')?.addEventListener('click', () => {
      queue.push({ type: 'redo', payload: null });
    });

    document.getElementById('fenButton')?.addEventListener('click', () => {
      queue.push({ type: 'restart', payload: null });
    });

    canvas.addEventListener('mousedown', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      queue.push({ type: 'mousedown', payload: { x, y } });
    });

    canvas.addEventListener('mousemove', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      queue.push({ type: 'mousemove', payload: { x, y } });
    });

    canvas.addEventListener('mouseup', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      queue.push({ type: 'mouseup', payload: { x, y } });
    });

    return true;
  }

  public tick(): void {
    let event : Event | undefined = undefined;
    while ((event = this.queue.shift()) !== undefined) {
      for (const listener of this.listeners) {
        listener.handleEvent(event);
      }
    }
  }

  public push(event: Event): void {
    this.queue.push(event);
    this.listeners.forEach(listener => listener.handleEvent(event));
  }
}
