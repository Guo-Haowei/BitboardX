use super::board::{BitBoard, Square};
use super::move_generator;
use super::moves::{Move, MoveFlags, create_move, validate_move};
use super::piece::{Color, Piece};
use super::utils::fen::*;

pub struct Position {
    /// Data used to serialize/deserialize FEN.
    pub bitboards: [BitBoard; Piece::COUNT],

    pub side_to_move: Color,
    pub castling: u8,
    // @TODO: en passant
    pub halfmove_clock: u32,
    pub fullmove_number: u32,

    /// Data can be computed from the FEN state.
    pub occupancies: [BitBoard; 3],
    pub attack_map: [BitBoard; Color::COUNT],
}

impl Position {
    pub fn new() -> Self {
        let bitboards = [
            BitBoard::from(0x000000000000FF00), // White Pawns
            BitBoard::from(0x0000000000000042), // White Knights
            BitBoard::from(0x0000000000000024), // White Bishops
            BitBoard::from(0x0000000000000081), // White Rooks
            BitBoard::from(0x0000000000000008), // White Queens
            BitBoard::from(0x0000000000000010), // White King
            BitBoard::from(0x00FF000000000000), // Black Pawns
            BitBoard::from(0x4200000000000000), // Black Knights
            BitBoard::from(0x2400000000000000), // Black Bishops
            BitBoard::from(0x8100000000000000), // Black Rooks
            BitBoard::from(0x0800000000000000), // Black Queens
            BitBoard::from(0x1000000000000000), // Black King
        ];

        let occupancies = calc_occupancies(&bitboards);
        let attack_map = [BitBoard::from(0x0000000000FF0000), BitBoard::from(0x0000FF0000000000)];

        Self {
            bitboards,
            side_to_move: Color::WHITE,
            castling: MoveFlags::KQkq,
            halfmove_clock: 0,
            fullmove_number: 1,
            occupancies,
            attack_map,
        }
    }

    pub fn from_parts(
        piece_placement: &str,
        side_to_move: &str,
        castling_rights: &str,
        _en_passant_target: &str,
        halfmove_clock: &str,
        fullmove_number: &str,
    ) -> Result<Self, &'static str> {
        let bitboards = parse_board(piece_placement)?;
        let side_to_move = match Color::parse(side_to_move) {
            Some(color) => color,
            None => return Err("Invalid side to move in FEN"),
        };
        let castling = parse_castling(castling_rights)?;
        let halfmove_clock = parse_halfmove_clock(halfmove_clock)?;
        let fullmove_number = parse_fullmove_number(fullmove_number)?;

        let occupancies = calc_occupancies(&bitboards);

        let mut pos = Self {
            bitboards,
            side_to_move,
            castling,
            halfmove_clock,
            fullmove_number,
            occupancies,
            attack_map: [BitBoard::new(); 2],
        };

        pos.attack_map = pos.calc_attack_map();
        Ok(pos)
    }

    pub fn from(fen: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN: must have 6 fields");
        }

        Self::from_parts(parts[0], parts[1], parts[2], parts[3], parts[4], parts[5])
    }

    pub fn update_cache(&mut self) {
        self.occupancies = calc_occupancies(&self.bitboards);
        self.attack_map = self.calc_attack_map();
    }

    pub fn change_side(&mut self) {
        self.side_to_move = self.side_to_move.opponent()
    }

    pub fn get_piece(&self, sq: Square) -> Piece {
        for i in 0..Piece::COUNT {
            if self.bitboards[i].test(sq.as_u8()) {
                return unsafe { std::mem::transmute(i as u8) };
            }
        }

        Piece::NONE
    }

    pub fn is_legal_move(&mut self, m: &Move) -> bool {
        validate_move(self, &m)
    }

    pub fn pseudo_legal_move(&self, sq: Square) -> BitBoard {
        if self.occupancies[self.side_to_move.as_usize()].test(sq.as_u8()) {
            return move_generator::pseudo_legal_move(self, sq);
        }
        BitBoard::new()
    }

    pub fn legal_move(&mut self, sq: Square) -> BitBoard {
        let mut pseudo_legal = self.pseudo_legal_move(sq);

        let mut bits = pseudo_legal.get();

        while bits != 0 {
            let dst_sq = bits.trailing_zeros();

            if let Some(m) = create_move(self, sq, Square(dst_sq as u8)) {
                if !self.is_legal_move(&m) {
                    pseudo_legal.unset(dst_sq as u8);
                }
            }

            bits &= bits - 1;
        }

        pseudo_legal
    }

    fn calc_attack_map_impl<const COLOR: u8, const START: u8, const END: u8>(&self) -> BitBoard {
        let mut attack_map = BitBoard::new();

        for i in START..=END {
            // pieces from W to B
            let bb = self.bitboards[i as usize];
            for sq in 0..64 {
                if bb.test(sq) {
                    attack_map |=
                        move_generator::pseudo_legal_attack(self, Square(sq), Color::from(COLOR));
                }
            }
        }

        attack_map
    }

    pub fn calc_attack_map(&self) -> [BitBoard; Color::COUNT] {
        [
            self.calc_attack_map_impl::<{ Color::WHITE.as_u8() }, { Piece::W_START }, { Piece::W_END }>(),
            self.calc_attack_map_impl::<{ Color::BLACK.as_u8() }, { Piece::B_START }, { Piece::B_END }>(),
        ]
    }

    pub fn to_string(&self, pad: bool) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            s.push((rank as u8 + b'1') as char);
            s.push(' ');
            for file in 0..8 {
                let sq = rank * 8 + file;
                let piece_char = if self.bitboards[Piece::W_PAWN.as_usize()].test(sq) {
                    '♙'
                } else if self.bitboards[Piece::W_KNIGHT.as_usize()].test(sq) {
                    '♘'
                } else if self.bitboards[Piece::W_BISHOP.as_usize()].test(sq) {
                    '♗'
                } else if self.bitboards[Piece::W_ROOK.as_usize()].test(sq) {
                    '♖'
                } else if self.bitboards[Piece::W_QUEEN.as_usize()].test(sq) {
                    '♕'
                } else if self.bitboards[Piece::W_KING.as_usize()].test(sq) {
                    '♔'
                } else if self.bitboards[Piece::B_PAWN.as_usize()].test(sq) {
                    '♟'
                } else if self.bitboards[Piece::B_KNIGHT.as_usize()].test(sq) {
                    '♞'
                } else if self.bitboards[Piece::B_BISHOP.as_usize()].test(sq) {
                    '♝'
                } else if self.bitboards[Piece::B_ROOK.as_usize()].test(sq) {
                    '♜'
                } else if self.bitboards[Piece::B_QUEEN.as_usize()].test(sq) {
                    '♛'
                } else if self.bitboards[Piece::B_KING.as_usize()].test(sq) {
                    '♚'
                } else {
                    '.'
                };

                if piece_char == '.' {
                    s.push('・');
                } else {
                    s.push(piece_char);
                    if pad {
                        s.push(' ');
                    }
                }
            }
            s.push('\n');
        }
        s.push_str("  ａｂｃｄｅｆｇｈ\n");
        s.push_str(format!("Side: {}\n", self.side_to_move).as_str());
        s.push_str(format!("Castling: {}\n", &castling_to_string(self.castling)).as_str());
        s.push_str(format!("Halfmove clock: {}\n", self.halfmove_clock).as_str());
        s.push_str(format!("Fullmove number: {}\n", self.fullmove_number).as_str());

        s
    }

    pub fn to_board_string(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let c = if self.bitboards[Piece::W_BISHOP.as_usize()].test(sq) {
                    'B'
                } else if self.bitboards[Piece::W_KNIGHT.as_usize()].test(sq) {
                    'N'
                } else if self.bitboards[Piece::W_PAWN.as_usize()].test(sq) {
                    'P'
                } else if self.bitboards[Piece::W_ROOK.as_usize()].test(sq) {
                    'R'
                } else if self.bitboards[Piece::W_QUEEN.as_usize()].test(sq) {
                    'Q'
                } else if self.bitboards[Piece::W_KING.as_usize()].test(sq) {
                    'K'
                } else if self.bitboards[Piece::B_BISHOP.as_usize()].test(sq) {
                    'b'
                } else if self.bitboards[Piece::B_KNIGHT.as_usize()].test(sq) {
                    'n'
                } else if self.bitboards[Piece::B_PAWN.as_usize()].test(sq) {
                    'p'
                } else if self.bitboards[Piece::B_ROOK.as_usize()].test(sq) {
                    'r'
                } else if self.bitboards[Piece::B_QUEEN.as_usize()].test(sq) {
                    'q'
                } else if self.bitboards[Piece::B_KING.as_usize()].test(sq) {
                    'k'
                } else {
                    '.'
                };
                s.push(c);
            }
        }
        s
    }
}

fn castling_to_string(castling: u8) -> String {
    let mut result = String::new();
    for (i, c) in ['K', 'Q', 'k', 'q'].iter().enumerate() {
        if castling & (1 << i) != 0 {
            result.push(*c);
        }
    }
    if result.is_empty() { "-".to_string() } else { result }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor_new() {
        let pos = Position::new();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::KQkq);
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_constructor_from_parts() {
        let pos = Position::from_parts(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        )
        .unwrap();
        assert!(pos.bitboards[Piece::W_PAWN.as_usize()].equal(0x000000000000FF00u64));
        assert!(pos.bitboards[Piece::B_PAWN.as_usize()].equal(0x00FF000000000000u64));
        assert!(pos.bitboards[Piece::W_ROOK.as_usize()].equal(0x0000000000000081u64));
        assert!(pos.bitboards[Piece::B_ROOK.as_usize()].equal(0x8100000000000000u64));

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::KQkq);
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_number, 1);
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );
    }

    #[test]
    fn test_constructor_from() {
        let pos = Position::from("r1bqk2r/pp1n1ppp/2pbpn2/8/3P4/2N1BN2/PPP2PPP/R2QKB1R w Kq - 6 7")
            .unwrap();

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, MoveFlags::K | MoveFlags::q);
        assert_eq!(pos.halfmove_clock, 6);
        assert_eq!(pos.fullmove_number, 7);
        assert_eq!(
            pos.to_board_string(),
            "r.bqk..rpp.n.ppp..pbpn.............P......N.BN..PPP..PPPR..QKB.R"
        );
    }

    #[test]
    fn test_calc_attack_map() {
        let pos =
            Position::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let attack_maps = pos.calc_attack_map();

        assert_eq!(attack_maps[Color::WHITE.as_usize()].get(), 0x0000000000FF0000);
        assert_eq!(attack_maps[Color::BLACK.as_usize()].get(), 0x0000FF0000000000);
    }

    #[test]
    fn test_trailing_zeros() {
        let mut bits: u64 = 0b10101000;
        let mut count = 0;
        let expect = [3, 5, 7];
        while bits != 0 {
            let tz = bits.trailing_zeros();
            bits &= bits - 1;
            assert_eq!(tz, expect[count]);
            count += 1;
        }
        assert_eq!(count, expect.len());
    }
}
