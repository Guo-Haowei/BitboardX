import { useEffect, useRef, useState } from "react";
import * as Chess from "./chess";

import ChessBoard from "./components/ChessBoard";
import PlayerSelector, {
  type PlayerType,
} from "./components/PlayerSelector";
import Sidebar from "./components/Sidebar";
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

  const [newGameOpen, setNewGameOpen] = useState(false);

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
    const canvas = canvasRef.current;

    if (!canvas) {
      return;
    }

    async function initialize(canvas: HTMLCanvasElement) {
      const chessController = await Chess.initialize({
        canvas,
      });

      setController(chessController);

      chessController.newGame(
        new Chess.UIPlayer(),
        new Chess.BotPlayer(),
        ""
      );
    }

    initialize(canvas);
  }, []);

  return (
    <>
      <TopBar />

      <main className="game-layout">
        <section className="game-column">
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
                    {whitePlayer === "bot"
                      ? "Bot"
                      : "Human"}
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
                    {blackPlayer === "bot"
                      ? "Bot"
                      : "Human"}
                  </div>
                </div>
              </div>
            </div>

            {/* Chess board */}
            <ChessBoard canvasRef={canvasRef} />

            {/* Temporary old new-game controls */}
            <PlayerSelector
              color="White"
              value={whitePlayer}
              onChange={setWhitePlayer}
            />

            <p />

            <PlayerSelector
              color="Black"
              value={blackPlayer}
              onChange={setBlackPlayer}
            />

            <p />

            <input
              type="text"
              id="fen-input"
              value={fen}
              onChange={(event) =>
                setFen(event.target.value)
              }
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

            {newGameOpen && (
              <p>
                New Game modal will go here next.
              </p>
            )}
          </div>
        </section>

        <Sidebar
          onNewGame={() => setNewGameOpen(true)}
          onUndo={() => controller?.undo()}
        />
      </main>
    </>
  );
}