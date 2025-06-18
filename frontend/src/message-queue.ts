/// GameManager receives gamestart, then request input from the players
/// Players send their moves to GameManager
/// AnimationManager receives the moves from GameManager, start to animate
/// and emits animation:done event
/// GameManager receives animation:done event, requests the next input from the players

class EventMap {
    public readonly NEW_GAME = 'newgame';
    public readonly GAME_OVER = 'gameover';
    public readonly REQUEST_PLAYER_INPUT = 'request-player-input';
    public readonly MOVE = 'move';
    public readonly ANIMATION_DONE = 'animation-done';
};

export const Message = new EventMap();

// const DEBUG = false;
const DEBUG = true;

export interface Listener {
    handleMessage(message: string): void;
};

class MessageQueue {
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

export const messageQueue = new MessageQueue();