export const BOARD_SIZE = 8;
export const TILE_SIZE = 80;

export const DEFAULT_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

export const COLORS: Record<string, string> = {
  green: '#008000',
  red: '#ff0000',
  light: '#f0d9b5',
  dark: '#b58863',
};

export const PIECE_SYMBOLS: Record<string, string> = {
  'r': '♜', 'n': '♞', 'b': '♝', 'q': '♛', 'k': '♚', 'p': '♟',
  'R': '♖', 'N': '♘', 'B': '♗', 'Q': '♕', 'K': '♔', 'P': '♙',
};
