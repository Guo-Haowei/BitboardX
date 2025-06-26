from dataclasses import dataclass
from pathlib import Path
import chess.pgn
import sys
import os

html_head = '''<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Chess Game Results</title>
  <style>
    body {
      font-family: sans-serif;
      max-width: 800px;
      margin: 2rem auto;
      padding: 1rem;
      background: #fdfdfd;
      color: #222;
    }
    h2 { margin-top: 2rem; }
    hr { margin: 2rem 0; }
    .bar-container {
      background: #eee;
      border: 1px solid #ccc;
      width: 200px;
      height: 16px;
      margin-bottom: 4px;
    }
    .bar-fill {
      height: 100%;
    }
    .stat-label {
      font-weight: bold;
    }
    .merged-bar {
      display: flex;
      height: 20px;
      width: 300px;
      border: 1px solid #aaa;
      margin: 8px 0;
    }

    .bar-segment {
      height: 100%;
    }

    .win  { background-color: #4caf50; }  /* Green */
    .draw { background-color: #9e9e9e; }  /* Gray */
    .loss { background-color: #f44336; }  /* Red */

    textarea {
      width: 100%;
      height: 200px;
      font-family: monospace;
      font-size: 14px;
    }
    button {
      margin-top: 10px;
      padding: 6px 12px;
    }
  </style>
</head>
<body>
<h1>Chess Game Report</h1>
'''

html_foot = '''
  <script>
    function copyPGN(num) {
      const textarea = document.getElementById("pgn-" + num);
      textarea.select();
      document.execCommand("copy");
    }
    function openAnalysis() {
      window.open("https://www.chess.com/analysis?tab=analysis", "_blank");
    }
  </script>
</body>
</html>
'''

@dataclass
class Stats:
    wins: int
    losses: int
    draws: int

def result_bar(wins, draws, losses):
    total = wins + draws + losses or 1  # avoid zero division
    return f'''
    <div class="merged-bar">
      <div class="bar-segment win" style="flex: {wins};"></div>
      <div class="bar-segment draw" style="flex: {draws};"></div>
      <div class="bar-segment loss" style="flex: {losses};"></div>
    </div>
    <div>
      Wins: {wins} &nbsp; Draws: {draws} &nbsp; Losses: {losses}
    </div>
    '''

def pgn_to_markdown(pgn_path, md_path):
    with open(pgn_path) as pgn, open(md_path, 'w') as html:
        game_count = 1

        stats = {}

        html.write(html_head)

        details = ''

        while True:
            game = chess.pgn.read_game(pgn)
            if game is None:
                break  # No more games

            white = game.headers.get('White')
            black = game.headers.get('Black')
            if not white in stats:
                stats[white] = Stats(0, 0, 0)
            if not black in stats:
                stats[black] = Stats(0, 0, 0)

            result = game.headers.get('Result')

            if result == '1-0':
                stats[white].wins += 1
                stats[black].losses += 1
                summary = f'{white} wins'
            elif result == '0-1':
                stats[white].losses += 1
                stats[black].wins += 1
                summary = f'{black} wins'
            elif result == '1/2-1/2':
                stats[white].draws += 1
                stats[black].draws += 1
                summary = 'Draw'

            details += f'<h2>Game {game_count}: {white} vs {black} ({summary})</h2>\n'
            details += f'<ul>\n'
            details += f'  <li><strong>Date</strong>: {game.headers.get("Date", "Unknown")}</li>\n'
            details += f'  <li><strong>Result</strong>: {result}</li>\n'
            details += f'</ul>\n'

            # Moves
            board = game.board()
            moves = []
            for i, move in enumerate(game.mainline_moves(), start=1):
                san = board.san(move)
                if i % 2 == 1:
                    moves.append(f'{(i+1)//2}. {san}')
                else:
                    moves[-1] += f' {san}'
                board.push(move)

            details += f'<textarea id="pgn-{game_count}" readonly>'
            details += f'[White "{white}"]\n'
            details += f'[Black "{black}"]\n'
            details += ' '.join(moves)
            details += '</textarea>\n'
            details += f'<button onclick="copyPGN({game_count})">Copy PGN</button>\n'
            details += f'<button onclick="openAnalysis()">Open Analysis</button>\n'
            details += '<hr>\n'
            game_count += 1

        for player, stats in stats.items():
            total = stats.wins + stats.losses + stats.draws
            if total > 0:
                html.write(f'<h2>{player} Results</h2>\n')
                html.write(result_bar(stats.wins, stats.draws, stats.losses))

        html.write(details)

        html.write(html_foot)

def main():
    if len(sys.argv) < 2:
        print('Usage: python report.py <path_to_pgn_file>')
        sys.exit(1)

    p = Path(sys.argv[1])
    new_name = p.with_suffix('.html').name  # replaces extension, keeps filename only

    pgn_to_markdown(sys.argv[1], new_name)

main()