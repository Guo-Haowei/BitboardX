export const BOARD_SIZE: number = 8;
export const TILE_SIZE: number = 60;

export const DEFAULT_FEN: string = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

export const COLORS: { [key: string]: string } = {
  green: '#008000',
  red: '#ff0000',
  light: '#f0d9b5',
  dark: '#b58863',
};

export const PIECE_SYMBOLS: { [key: string]: string } = {
  'r': '♜', 'n': '♞', 'b': '♝', 'q': '♛', 'k': '♚', 'p': '♟',
  'R': '♖', 'N': '♘', 'B': '♗', 'Q': '♕', 'K': '♔', 'P': '♙',
};
