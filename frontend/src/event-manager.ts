import { RuntimeModule, runtime } from './runtime';
import { picker } from './picker';
import { Message, messageQueue } from './message-queue';

// @TODO: depracate this interface
export class EventManager implements RuntimeModule {
  public getName(): string {
    return 'EventManager';
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
      messageQueue.emit(Message.NEW_GAME);
    });

    canvas.addEventListener('mouseup', (e) => {
      const { x, y } = getMousePosition(canvas, e);
      picker.onMouseUp(x, y);
    });

    return true;
  }

  public tick(): void {
    // let event: Event | undefined = undefined;
    // while ((event = this.queue.shift()) !== undefined) {
    //   for (const listener of this.listeners) {
    //     listener.handleEvent(event);
    //   }
    // }
  }
}
