import { RuntimeModule } from "./runtime";
import { BOARD_SIZE } from "./constants";

export class Display implements RuntimeModule {
  private _canvas: HTMLCanvasElement;
  private _tileSize = 0;

  public constructor() {
    const canvas = document.getElementById('chessCanvas') as HTMLCanvasElement;
    this._canvas = canvas;
    // const container = document.getElementById('left-column');
    // container?.appendChild(canvas);
    canvas.tabIndex = 0;
    canvas.style.margin = '20px auto';

    const minSize = 800;
    this.onResize(minSize);
  }

  public get canvas(): HTMLCanvasElement {
    return this._canvas;
  }

  public get tileSize(): number {
    return this._tileSize;
  }

  public init(): boolean {
    return true;
  }

  public tick() {
    // This method is intentionally left empty.
  }

  public onResize(minSize = 200) {
    const canvas = this._canvas;

    let size = Math.min(canvas.clientWidth, canvas.clientHeight);
    size = Math.max(size, minSize);
    canvas.width = size;
    canvas.height = size;

    this._tileSize = size / (BOARD_SIZE + 1);
    console.log(`tile size is ${this._tileSize}`);
  }
}