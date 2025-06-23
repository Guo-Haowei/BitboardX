async function loadMeta() {
  try {
    const response = await fetch('http://localhost:3000/meta');
    if (!response.ok) throw new Error(`HTTP error ${response.status}`);
    console.log(`Response status: ${response.status}`);

    const data = await response.json(); // assuming it's an array of { player1, player2, result, file }
    console.log(data);

    // data.forEach(match => {
    //   console.log(match);
    // });
  } catch (err) {
    console.error('Failed to load meta:', err);
  }
}

window.onload = loadMeta;

const canvas = document.getElementById('chessboard') as HTMLCanvasElement;
const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
const squareSize = canvas.width / 8;
function drawBoard() {
  for (let y = 0; y < 8; y++) {
    for (let x = 0; x < 8; x++) {
      const isLight = (x + y) % 2 === 0;
      ctx.fillStyle = isLight ? '#f0d9b5' : '#b58863';
      ctx.fillRect(x * squareSize, y * squareSize, squareSize, squareSize);
    }
  }
}
drawBoard();