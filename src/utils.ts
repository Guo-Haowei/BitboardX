export type Point2D = {
  x: number;
  y: number;
};

export const isLowerCase = (char: string): boolean => {
  return char === char.toLowerCase() && char !== char.toUpperCase();
};
