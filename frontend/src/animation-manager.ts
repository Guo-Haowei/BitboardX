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

export class AnimationManager implements RuntimeModule {
    private _animations: Animation[];
    private lastTime: number;

    public constructor() {
        this._animations = [];
        this.lastTime = 0;
    }

    public init(): boolean {
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
    }

    private addCastlingAnimation(src: string, dst: string) {
        if (src === 'e1') {
            if (dst === 'g1') {
                this.addAnimation('h1', 'f1'); // Rook moves from h1 to f1
            } else if (dst === 'c1') {
                this.addAnimation('a1', 'd1'); // Rook moves from a1 to d1
            }
        } else if (src === 'e8') {
            if (dst === 'g8') {
                this.addAnimation('h8', 'f8'); // Rook moves from h8 to f8
            } else if (dst === 'c8') {
                this.addAnimation('a8', 'd8'); // Rook moves from a8 to d8
            }
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

        // @TODO: if castling, we need to animate the rook as well

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
