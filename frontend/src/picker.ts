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
            console.log(`board: ${board.board}`);
            console.log(`Selected piece: ${this._piece} at square: ${this._square}`);
        } else {
            this.reset();
        }
    }

    private sendMove(dst: string) {
        const src = this._square;
        const dstRank = parseInt(dst[1], 10);
        const white_promo = (dstRank === 8) && (this._piece === 'P');
        const black_promo = (dstRank === 1) && (this._piece === 'p');
        if (white_promo || black_promo) {
            console.log(`Promoting ${this._piece} at ${src} to a different piece.`);
            console.log(`Dest is is ${white_promo ? 'white' : 'black'} at rank ${dstRank}.`);
            console.log(`moving from ${src} to ${dst}.`);
        }
        const promo = (white_promo || black_promo) ? (prompt("Enter what piece to promote: ") || '') : '';
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
