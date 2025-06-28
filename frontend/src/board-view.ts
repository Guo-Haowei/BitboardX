import { ChessBoard } from './chess';

export abstract class BoardView {
  protected onclickCallback?: ((square: string) => void);

  setOnClickCallback(callback: (square: string) => void) {
    this.onclickCallback = callback;
  }

  abstract draw(board: ChessBoard, selected?: string): void;
}
