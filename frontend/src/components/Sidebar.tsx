interface Props {
  onNewGame: () => void;
  onUndo: () => void;
}

export default function Sidebar({
  onNewGame,
  onUndo,
}: Props) {
  return (
    <aside className="sidebar">
      <section className="panel">
        <h2>Game Info</h2>

        <div className="game-status">
          <div className="status-line">
            <span className="status-dot" />
            <span>Your turn</span>
          </div>

          <div>White to move</div>

          <div className="status-description">
            Good luck!
          </div>
        </div>
      </section>

      <section className="panel move-history-panel">
        <h2>Move History</h2>

        <div className="move-history">
          <div className="move-row">
            <span className="move-number">1.</span>
            <span>—</span>
            <span>—</span>
          </div>
        </div>
      </section>

      <section className="panel controls-panel">
        <button
          className="sidebar-button primary"
          onClick={onNewGame}
        >
          ⊕ New Game
        </button>

        <div className="sidebar-control-row">
          <button
            className="sidebar-button secondary"
            onClick={onUndo}
          >
            ↶ Undo
          </button>

          <button
            className="sidebar-button danger"
            disabled
          >
            ⚑ Resign
          </button>
        </div>
      </section>

      <section className="panel">
        <h2>Settings</h2>

        <div className="setting-row">
          <span>⇄ Flip Board</span>
          <input
            type="checkbox"
            disabled
          />
        </div>

        <div className="setting-row">
          <span>🔊 Sound</span>
          <input
            type="checkbox"
            defaultChecked
            disabled
          />
        </div>

        <div className="setting-row">
          <span>◎ Highlight Legal Moves</span>
          <input
            type="checkbox"
            defaultChecked
            disabled
          />
        </div>
      </section>
    </aside>
  );
}