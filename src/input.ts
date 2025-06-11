import { Point2D } from "./utils";

type InputEvent = {
  type: string;
  payload?: Point2D | undefined;
};

export class InputManager {
  public eventQueue: InputEvent[];

  public constructor() {
    this.eventQueue = [];
  }

  public init(canvas: HTMLCanvasElement): boolean {
    const getMousePosition = (canvas: HTMLCanvasElement, e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      return {x, y};
    };

    const { eventQueue } = this;

    document.getElementById('undoButton')?.addEventListener('click', () => {
      eventQueue.push({ type: 'undo' });
    });

    document.getElementById('redoButton')?.addEventListener('click', () => {
      eventQueue.push({ type: 'redo' });
    });

    canvas.addEventListener('mousedown', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      eventQueue.push({ type: 'mousedown', payload: { x, y } });
    });

    canvas.addEventListener('mousemove', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      eventQueue.push({ type: 'mousemove', payload: { x, y } });
    });

    canvas.addEventListener('mouseup', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      eventQueue.push({ type: 'mouseup', payload: { x, y } });
    });

    return true;
  }
}

export const inputManager: InputManager = new InputManager();