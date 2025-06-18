import { runtime } from './runtime';
import { fileRankToSquare } from './utils';
import { TILE_SIZE } from './constants';

class Picker {
    private _square: string;
    private _moves: Set<string> | undefined;
    private _piece: string;

    public constructor() {
        this.reset();
    }

    private reset() {
        this._square = '';
        this._piece = '';
        this._moves = undefined;
    }

    public onMouseUp(x: number, y: number): void {
        const file = Math.floor(x / TILE_SIZE);
        const rank = 7 - Math.floor(y / TILE_SIZE);
        const square = fileRankToSquare(file, rank);
        const { board } = runtime.gameManager;
        if (this._moves && this._moves.has(square)) {
            this.sendMove(square);
            return;
        }

        const moves = board.legalMoves(square);
        if (moves !== undefined) {
            this._moves = moves;
            this._square = square;
            this._piece = board.board[file + rank * 8];
        } else {
            this.reset();
        }
    }

    private sendMove(dst: string) {
        const src = this._square;
        const dstRank = parseInt(dst[1], 10);
        let promo = '';
        let is_promo = dstRank === 8 && this._piece === 'P';
        is_promo = is_promo || (dstRank === 1 && this._piece === 'p');
        if (is_promo) {
            promo = prompt("Enter what piece to promote: ") || '';
        }

        const move = `${src}${dst}${promo}`;
        runtime.gameManager.injectMove(move);

        this.reset();
    }

    public get square() {
        return this._square;
    }

    public get moves() {
        return this._moves;
    }

    public get piece() {
        return this._piece;
    }
}

export const picker = new Picker();
