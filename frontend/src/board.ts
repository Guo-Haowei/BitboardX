export class Board {
    private _board = '';
    private _moveLookup = new Map<string, Set<string>>();

    public set(board: string, legalMoves: string[]) {
        if (board === this._board) {
            return false;
        }

        this._board = board;
        this._moveLookup.clear();

        legalMoves.forEach((move) => {
            const from = move.slice(0, 2);
            const to = move.slice(2, 4);
            if (!this._moveLookup.has(from)) {
                this._moveLookup.set(from, new Set());
            }
            this._moveLookup.get(from)?.add(to);
        });

        return true;
    }

    public get board() {
        return this._board;
    }

    public legalMoves(square: string): Set<string> | undefined {
        return this._moveLookup.get(square);
    }
}
