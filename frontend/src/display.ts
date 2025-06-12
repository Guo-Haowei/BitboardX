import { BOARD_SIZE, TILE_SIZE } from './constants';
import { RuntimeModule } from "./runtime";

export class Display implements RuntimeModule {
  private _canvas: HTMLCanvasElement;

  public constructor() {
    const canvas = document.createElement('canvas');
    canvas.width = TILE_SIZE * (BOARD_SIZE + 1);
    canvas.height = TILE_SIZE * (BOARD_SIZE + 1);
    canvas.tabIndex = 0;
    canvas.style = 'position: absolute; top: 0px; left: 0px; right: 0px; bottom: 0px; margin: auto;';
    document.body.appendChild(canvas);
    this._canvas = canvas;
  }

  public get canvas(): HTMLCanvasElement {
    return this._canvas;
  }

  public getName(): string {
    return 'Display';
  }

  public init(): boolean {
    return true;
  }

  public tick() {
  }
}