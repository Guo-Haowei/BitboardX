import { Listener, Message, messageQueue } from "./message-queue";
import { RuntimeModule } from "./runtime";

export class AnimationManager implements RuntimeModule, Listener {
    private counter = 0;

    public constructor() {
        // Subscribe to the 'move' event to handle animations
        messageQueue.subscribe(Message.MOVE, this);
    }

    public getName(): string {
        return 'AnimationManager';
    }

    public init(): boolean {
        // Initialization logic if needed
        return true;
    }

    public tick(): void {
        if (this.counter > 0) {
            this.counter--;
            if (this.counter === 0) {
                messageQueue.emit(`${Message.ANIMATION_DONE}:`);
            }
        }
    }

    public handleMessage(message: string): void {
        const [event, ...payload] = message.split(':');
        switch (event) {
            case Message.MOVE: this.counter = 10; break;
            default: break;
        }
    }
};
