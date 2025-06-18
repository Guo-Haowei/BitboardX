interface FenParts {
  board: string;
  activeColor?: string;
  castlingAvailability?: string;
  enPassantTargetSquare?: string;
  halfMoveClock?: number;
  fullMoveNumber?: number;
}

function parseBoard(board: string) {
  let result = '';
  const rows = board.split('/').reverse();
  for (const row of rows) {
    for (const char of row) {
      const emptyFiles = char.charCodeAt(0) - 48; // Convert char to number
      if (emptyFiles >= 1 && emptyFiles <= 8) {
        result += '.'.repeat(emptyFiles);
      } else {
        result += char;
      }
    }
  }

  // console.log(`Parsed board: ${result}`);
  return result;
}

function parseFen(fen: string): FenParts {
  const [boardStr] = fen.split(' ');
  const board = parseBoard(boardStr);

  return { board };
}

export class Board {
  private _fen = '';
  private _board = '';
  private _moveLookup = new Map<string, Set<string>>();

  public set(fen: string, legalMoves: string[]) {
    if (fen === this._fen) {
      return false;
    }

    const { board } = parseFen(fen);

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
