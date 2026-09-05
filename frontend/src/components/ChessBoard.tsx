import type { RefObject } from "react";

interface Props {
  canvasRef: RefObject<HTMLCanvasElement | null>;
}

export default function ChessBoard({ canvasRef }: Props) {
  return (
    <div className="board-container">
      <canvas
        ref={canvasRef}
        id="chess-board"
        width={320}
        height={320}
        tabIndex={0}
      />
    </div>
  );
}