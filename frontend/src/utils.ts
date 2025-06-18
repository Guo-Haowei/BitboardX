// utils.ts

export function isLowerCase(char: string) {
  return char === char.toLowerCase() && char !== char.toUpperCase();
};

export function fileRankToSquare(file: number, rank: number) {
  return `${String.fromCharCode(97 + file)}${rank + 1}`;
};

export function squareToFileRank(square: string): [number, number] {
  const file = square.charCodeAt(0) - 97; // 'a' is 97 in ASCII
  const rank = square.charCodeAt(1) - 49; // '1' is 49 in ASCII, so we subtract 1 to get zero-based rank
  return [file, rank];
}