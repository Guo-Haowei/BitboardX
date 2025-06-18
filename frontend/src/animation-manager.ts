import { Listener, Message } from "./message-queue";
import { runtime, RuntimeModule } from "./runtime";

export class AnimationManager implements RuntimeModule, Listener {
    private counter: number;
    private _playing: boolean;

    public constructor() {
        this.counter = 0;
        this._playing = false;
    }

    public getName(): string {
        return 'AnimationManager';
    }

    public init(): boolean {
        // Initialization logic if needed
        runtime.messageQueue.subscribe(Message.MOVE, this);
        return true;
    }

    public tick(): void {
        if (this.counter > 0) {
            this.counter--;
            if (this.counter === 0) {
                this._playing = false;
                runtime.messageQueue.emit(`${Message.ANIMATION_DONE}:`);
            }
        }
    }

    public handleMessage(message: string): void {
        const [event, move] = message.split(':');
        switch (event) {
            case Message.MOVE: {
                console.log(`AnimationManager: received move ${move}`);
                this.counter = 10;
                this._playing = true;
            } break;
            default: break;
        }
    }

    public get playing() {
        return this._playing;
    }
};
