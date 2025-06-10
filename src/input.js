
export class InputManager {
  constructor() {
    this.eventQueue = [];
  }

  init(canvas) {
    const getMousePosition = (canvas, e) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      return {x, y};
    };

    const { eventQueue } = this;

    document.getElementById('undoButton').addEventListener('click', () => {
      eventQueue.push({ type: 'undo' });
    });

    document.getElementById('redoButton').addEventListener('click', () => {
      eventQueue.push({ type: 'redo' });
    });

    canvas.addEventListener('mousedown', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      eventQueue.push({ type: 'mousedown', x, y });
    });

    canvas.addEventListener('mousemove', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      eventQueue.push({ type: 'mousemove', x, y});
    });

    canvas.addEventListener('mouseup', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      eventQueue.push({ type: 'mouseup', x, y });
    });
  }
}
