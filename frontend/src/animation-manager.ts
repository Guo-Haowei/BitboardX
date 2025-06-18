import { Listener, Message } from "./message-queue";
import { runtime, RuntimeModule } from "./runtime";
import { squareToFileRank } from "./utils";

export interface Animation {
    piece: string;
    dstFile: number;
    dstRank: number;
    dy: number;
    dx: number;
    x: number;
    y: number;
    duration: number;
    timeLeft: number;
}

export class AnimationManager implements RuntimeModule, Listener {
    private _animations: Animation[];
    private lastTime: number;

    public constructor() {
        this._animations = [];
        this.lastTime = 0;
    }

    public init(): boolean {
        runtime.messageQueue.subscribe(Message.MOVE, this);
        this.lastTime = Date.now();
        return true;
    }

    public tick(): void {
        const now = Date.now();
        const delta = now - this.lastTime;
        this.lastTime = now;

        if (this._animations.length === 0) {
            return;
        }

        const filtered = this._animations.filter(animation => {
            animation.timeLeft -= delta;
            if (animation.timeLeft <= 0) {
                return false;
            }

            animation.x += animation.dx * delta;
            animation.y += animation.dy * delta;
            return true;
        });

        this._animations = filtered;
        if (this._animations.length === 0) {
            runtime.messageQueue.emit(`${Message.ANIMATION_DONE}:`);
        }
    }

    public handleMessage(message: string): void {
        const [event, move] = message.split(':');
        switch (event) {
            case Message.MOVE: {
                this.addAnimation(move.slice(0, 2), move.slice(2, 4));
            } break;
            default: break;
        }
    }

    public get animations() {
        return this._animations;
    }

    private addAnimation(src: string, dst: string) {
        const [file, rank] = squareToFileRank(src);
        const [dstFile, dstRank] = squareToFileRank(dst);
        const idx = dstFile + dstRank * 8;
        const piece = runtime.gameManager.board.board[idx];

        const dist = Math.sqrt((dstFile - file) ** 2 + (dstRank - rank) ** 2);
        const duration = 300 * dist; // Duration in milliseconds
        const timeLeft = duration;

        const x = file;
        const y = 7 - rank;
        const dy = -(dstRank - rank) / duration;
        const dx = (dstFile - file) / duration;

        const animation: Animation = {
            piece,
            x,
            y,
            dy,
            dx,
            dstFile,
            dstRank,
            duration,
            timeLeft
        };

        this._animations.push(animation);
    }
};
