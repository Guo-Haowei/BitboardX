import { useEffect, useRef, useState } from "react";
import * as Chess from "./chess";
import ChessBoard from "./components/ChessBoard";
import PlayerSelector, { PlayerType } from "./components/PlayerSelector";
import TopBar from "./components/TopBar";

type ChessController = Awaited<
  ReturnType<typeof Chess.initialize>
>;

export default function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [controller, setController] =
    useState<ChessController | null>(null);

  const [whitePlayer, setWhitePlayer] =
    useState<PlayerType>("human");

  const [blackPlayer, setBlackPlayer] =
    useState<PlayerType>("bot");

  const [fen, setFen] = useState("");

  function createPlayer(type: PlayerType): Chess.Player {
    if (type === "bot") {
      return new Chess.BotPlayer();
    }

    return new Chess.UIPlayer();
  }

  function startGame() {
    if (!controller) {
      return;
    }

    const white = createPlayer(whitePlayer);
    const black = createPlayer(blackPlayer);

    controller.newGame(white, black, fen);
  }

  useEffect(() => {
    if (!canvasRef.current) {
      return;
    }

    async function initialize() {
      const chessController = await Chess.initialize({
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        canvas: canvasRef.current!,
      });

      setController(chessController);

      chessController.newGame(
        new Chess.UIPlayer(),
        new Chess.BotPlayer(),
        ""
      );
    }

    initialize();
  }, []);

  return (
    <>
      <TopBar />
      <div className="chess-app">
        {/* Header */}
        <div className="header">
          <div className="player you">
            <img
              className="avatar"
              src="https://lichess1.org/assets/piece/cburnett/wK.svg"
              alt="White"
            />

            <div className="info">
              <div>
                {whitePlayer === "bot" ? "Bot" : "Human"}
              </div>
            </div>
          </div>

          <div className="player opp">
            <img
              className="avatar"
              src="https://lichess1.org/assets/piece/cburnett/bK.svg"
              alt="Black"
            />

            <div className="info">
              <div>
                {blackPlayer === "bot" ? "Bot" : "Human"}
              </div>
            </div>
          </div>
        </div>

        {/* Chess board */}
        <ChessBoard canvasRef={canvasRef} />

        {/* Controls */}
        <div className="controls">
          <button
            id="resume-button"
            onClick={() => controller?.resume()}
          >
            Resume
          </button>

          <button
            id="undo-button"
            onClick={() => controller?.undo()}
          >
            Undo
          </button>
        </div>

        <PlayerSelector color="White" value={whitePlayer} onChange={setWhitePlayer} />
        <p />
        <PlayerSelector color="Black" value={blackPlayer} onChange={setBlackPlayer} />

        <p />

        {/* FEN */}
        <input
          type="text"
          id="fen-input"
          value={fen}
          onChange={(event) => setFen(event.target.value)}
          style={{ width: 300 }}
          placeholder="please enter FEN here"
        />

        <p />

        <button
          id="start-button"
          onClick={startGame}
          disabled={!controller}
        >
          Go!
        </button>
      </div>
    </>
  );
}