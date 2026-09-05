export default function TopBar() {
  return (
    <header className="topbar">
      <div className="brand">
        <span>Bitboard</span>
        <span className="brand-accent">X</span>
      </div>

      <nav className="nav">
        <a href="#" className="nav-link active">
          Play
        </a>

        <a
          href="https://github.com/Guo-Haowei/chess"
          className="nav-link"
          target="_blank"
          rel="noreferrer"
        >
          GitHub
        </a>
      </nav>

      <div className="topbar-actions">
        <button className="icon-button" title="Theme">
          ☀
        </button>

        <button className="icon-button" title="Sound">
          🔊
        </button>

        <button className="icon-button" title="Settings">
          ⚙
        </button>
      </div>
    </header>
  );
}