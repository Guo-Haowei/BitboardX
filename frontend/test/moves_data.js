export const MOVES_DATA =
{
  "Basic Movement": [
    [
      "r2q1rk1/pp3ppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10",
      {
        description: "Black pawn moves forward one square",
        moves: "d5d4"
      },
      {
        description: "Black pawn captures white pawn",
        moves: "d5e4"
      },
      {
        description: "Black pawn can't capture empty square",
        moves: "d5c4",
        invalid: "d5c4",
      },
      {
        description: "Black bishop captures white pawn",
        moves: "c5f2"
      },
      {
        description: "Black bishop can't capture piece behind pin",
        moves: "c5g1",
        invalid: "c5g1",
      },
      {
        description: "Knight can move to empty cells",
        moves: "f6g4 f3d4 g4e3"
      },
      {
        description: "Knight can capture",
        moves: "f6e4"
      },
      {
        description: "Knight can't move to cells occupied by own pieces",
        moves: "f6h7",
        invalid: "f6h7",
      }
    ]
  ],
  "En Passant": [
    [
      // 8 k . . . . . . .
      // 7 . . . . . . . .
      // 6 . . . . . . . .
      // 5 . p P . . . . .
      // 4 . . . . . p . .
      // 3 . . . . . . . .
      // 2 . . . . P . . .
      // 1 . . . . . . . K
      //   a b c d e f g h
      "k7/8/8/1pP5/5p2/8/4P3/7K w - b6 0 1",
      {
        description: "White pawn can capture black pawn at b6",
        moves: "c5b6"
      },
      {
        description: "Black pawn can capture white pawn at e3",
        moves: "e2e4 f4e3"
      },
      {
        description: "White pawn can't capture black pawn at b6 after one move",
        moves: "e2e4 f4e3 d5d7",
        invalid: "d5d7",
      },
      {
        description: "Black pawn can't capture white pawn if it pushes by one square (set up)",
        moves: "e2e3 a8a7 e3e4"
      },
      {
        description: "Black pawn can't capture white pawn if it pushes by one square",
        moves: "e2e3 a8a7 e3e4 f4e3",
        invalid: "f4e3",
      },
    ]
  ],
  "Castling": [
    [
      "r3k2r/pppbqppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPPBQPPP/R3K2R w KQkq - 0 1",
      {
        description: "White king can castle either side (Kingside)",
        moves: "e1g1"
      },
      {
        description: "White king can castle either side (Queenside)",
        moves: "e1c1"
      }
    ],
    [
      "r3k2r/pppbqppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPPBQPPP/R3K2R b KQkq - 0 1",
      {
        description: "Black king can castle either side (Kingside)",
        moves: "e8g8"
      },
      {
        description: "Black king can castle either side (Queenside)",
        moves: "e8c8"
      }
    ],
    [
      "r2bk2r/8/4B3/8/8/4b3/8/R3K2R w KQkq - 0 1",
      {
        description: "White king can't castle kingside because g1 is under attack",
        moves: "e1g1",
        invalid: "e1g1"
      },
      {
        description: "White king can't castle queenside because c1 is under attack",
        moves: "e1c1",
        invalid: "e1c1"
      }
    ],
    [
      "r2nk2r/8/4B3/8/8/4b3/8/R3K2R b KQkq - 0 1",
      {
        description: "Black king can't castle kingside because g8 is under attack",
        moves: "e8g8",
        invalid: "e8g8"
      },
      {
        description: "Black king can't castle queenside because d8 is blocked",
        moves: "e8c8",
        invalid: "e8c8"
      }
    ],
    {
      description: "White king can't castle because it's under attack",
      fen: "r2bk2r/8/4B3/8/8/8/3b4/R3K2R w KQkq - 0 1",
      moves: "e1c1",
      invalid: "e1c1"
    },
    {
      description: "White king can't castle kingside because rook is taken out",
      fen: "r2bk2r/8/4B3/8/4b3/8/8/R3K2R b KQkq - 0 1",
      moves: "e4h1 e1g1",
      invalid: "e1g1"
    },
    {
      description: "White king can't castle kingside, because castling is not allowed from fen",
      fen: "r2bk2r/8/4B3/8/4b3/8/8/R3K2R w kq - 0 1",
      moves: "e1g1",
      invalid: "e1g1"
    },
    {
      description: "White king can castle kingside, because rook is not taken out",
      fen: "r2bk2r/8/4B3/8/4b3/8/8/R3K2R w KQkq - 0 1",
      moves: "e1g1"
    }
  ]
};