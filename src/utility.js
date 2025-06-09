import { BOARD_SIZE, PIECE_SYMBOLS } from './constants.js';

export const isLowerCase = (char) => char === char.toLowerCase() && char !== char.toUpperCase();

export function printBoard(boardString) {
  let result = '';
  for (let y = 0; y < BOARD_SIZE; ++y) {
    let line = `${8 - y} `;
    for (let x = 0; x < BOARD_SIZE; ++x) {
      const intex = x + y * BOARD_SIZE;
      const c = boardString[intex];
      if (c === '.') {
        line += '・ ';
        continue;
      }
      line += PIECE_SYMBOLS[c] + ' ';
    }
    result += `${line}\n`;
  }
  result += "  ａ ｂ ｃ ｄ ｅ ｆ ｇ ｈ\n\n";
  console.log(result);
}