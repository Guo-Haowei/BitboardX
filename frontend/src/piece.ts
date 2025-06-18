// import { BOARD_SIZE, TILE_SIZE, PIECE_SYMBOLS } from './constants';
// import { isLowerCase } from './utils';

// export class Piece {
//     type: string;
//     x: number;
//     y: number;

//     public draw(ctx: CanvasRenderingContext2D) {
//         const piece = PIECE_SYMBOLS[this.type];

//         const x = this.x * TILE_SIZE + TILE_SIZE / 2;
//         const y = this.y * TILE_SIZE + TILE_SIZE / 2;
//         ctx.fillStyle = isLowerCase(this.type) ? 'black' : 'white';
//         ctx.fillText(piece, x, y);
//     }
// };

// export class Board {
//     private pieces: Piece[] = [];

//     public constructor(fen: string) {
//         console.log('Initializing board with FEN:', fen);
//         // Parse the FEN string and initialize the board with pieces
//     }

//     public draw(ctx: CanvasRenderingContext2D) {
//         // @TODO: Implement the drawing logic for the chessboard and pieces

//         // Draw the board
//         for (let rank = 7; rank >= 0; --rank) {
//             for (let file = 0; file < 8; ++file) {
//                 const color = ((rank + file) % 2 === 0 ? '#f0d9b5' : '#b58863');
//                 ctx.fillStyle = color;
//                 ctx.fillRect(file * TILE_SIZE, rank * TILE_SIZE, TILE_SIZE, TILE_SIZE);
//             }
//         }
//         // for (let row = 0; row < BOARD_SIZE; row++) {
//         // }
//     }
// };
