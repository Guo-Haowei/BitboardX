/* eslint-disable @typescript-eslint/no-unused-vars */
import { WasmMove } from '../../pkg/bitboard_x';
import { ChessBoard } from './chess';

export abstract class BoardView {
  protected onclickCallback?: ((square: string) => void);

  setOnClickCallback(callback: (square: string) => void) {
    this.onclickCallback = callback;
  }

  abstract update(board: ChessBoard, selected?: string): void;
  abstract addAnimation(piece: string, move: WasmMove): void;
  abstract hasAnimations(): boolean;
}

export class NullBoardView extends BoardView {
  update(board: ChessBoard, selected?: string) {
    console.error('NullBoardView.draw called, this should not happen');
  }

  addAnimation(piece: string, move: WasmMove) {
    console.error('NullBoardView.addAnimation called, this should not happen');
  }

  hasAnimations() {
    console.error('NullBoardView.hasAnimations called, this should not happen');
    return false;
  }
}
