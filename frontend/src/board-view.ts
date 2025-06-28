import { ChessBoard } from './chess';

export abstract class BoardView {
  protected onclickCallback?: ((square: string) => void);

  setOnClickCallback(callback: (square: string) => void) {
    this.onclickCallback = callback;
  }

  abstract draw(board: ChessBoard, selected?: string): void;
}

export class NullBoardView extends BoardView {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  draw(board: ChessBoard, selected?: string): void {
    // Do nothing
  }
}
