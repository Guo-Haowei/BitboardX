export const isLowerCase = (char: string): boolean => {
  return char === char.toLowerCase() && char !== char.toUpperCase();
};

export const fileRankToSquare = (file: number, rank: number): string => {
  return `${String.fromCharCode(97 + file)}${rank + 1}`;
};
