use super::super::board::{BitBoard, Square, constants::*};
use super::super::moves::*;
use super::super::piece::{Color, Piece, PieceType};
use super::super::position::Position;

const NORTH: i32 = 8;
const SOUTH: i32 = -NORTH;
const EAST: i32 = 1;
const WEST: i32 = -EAST;
const NE: i32 = NORTH + EAST;
const NW: i32 = NORTH + WEST;
const SE: i32 = SOUTH + EAST;
const SW: i32 = SOUTH + WEST;

const BOUND_A: BitBoard = BitBoard::from(0x0101010101010101);
const BOUND_B: BitBoard = BitBoard::from(0x0202020202020202);
const BOUND_G: BitBoard = BitBoard::from(0x4040404040404040);
const BOUND_H: BitBoard = BitBoard::from(0x8080808080808080);
const BOUND_1: BitBoard = BitBoard::from(0x00000000000000FF);
const BOUND_2: BitBoard = BitBoard::from(0x000000000000FF00);
const BOUND_7: BitBoard = BitBoard::from(0x00FF000000000000);
const BOUND_8: BitBoard = BitBoard::from(0xFF00000000000000);
const BOUND_AB: BitBoard = BitBoard::from(BOUND_A.get() | BOUND_B.get());
const BOUND_GH: BitBoard = BitBoard::from(BOUND_G.get() | BOUND_H.get());
const BOUND_12: BitBoard = BitBoard::from(BOUND_1.get() | BOUND_2.get());
const BOUND_78: BitBoard = BitBoard::from(BOUND_7.get() | BOUND_8.get());

fn shift(bb: BitBoard, dir: i32) -> BitBoard {
    // if dir > 0 { bb.get() << dir } else { bb.get() >> -dir }
    BitBoard::from(if dir < 0 { bb.get() >> -dir } else { bb.get() << dir })
}

fn shift_east(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_H).shift(EAST)
}

fn shift_west(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_A).shift(WEST)
}

fn shift_north(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_8).shift(NORTH)
}

fn shift_south(bb: BitBoard) -> BitBoard {
    (bb & !BOUND_1).shift(SOUTH)
}

fn shift_ne(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_H | BOUND_8)).shift(NE)
}

fn shift_nw(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_A | BOUND_8)).shift(NW)
}

fn shift_se(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_H | BOUND_1)).shift(SE)
}

fn shift_sw(bb: BitBoard) -> BitBoard {
    (bb & !(BOUND_A | BOUND_1)).shift(SW)
}

const SHIFT_FUNCS: [fn(BitBoard) -> BitBoard; 8] =
    [shift_north, shift_south, shift_east, shift_west, shift_ne, shift_nw, shift_se, shift_sw];

/// Pseudo-legal move generation for a square
fn pseudo_legal_from_impl<const ATTACK_ONLY: bool>(
    pos: &Position,
    sq: Square,
    color: Color,
) -> BitBoard {
    let piece = pos.get_piece(sq);

    match piece {
        Piece::W_PAWN => move_pawn::<{ Color::WHITE.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::B_PAWN => move_pawn::<{ Color::BLACK.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::W_ROOK | Piece::B_ROOK => move_sliding::<0, 4>(pos, sq, color),
        Piece::W_BISHOP | Piece::B_BISHOP => move_sliding::<4, 8>(pos, sq, color),
        Piece::W_QUEEN | Piece::B_QUEEN => move_sliding::<0, 8>(pos, sq, color),
        Piece::W_KNIGHT | Piece::B_KNIGHT => move_knight(pos, sq, color),
        Piece::W_KING => move_king::<{ Color::WHITE.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::B_KING => move_king::<{ Color::BLACK.as_u8() }, ATTACK_ONLY>(pos, sq),
        Piece::NONE => BitBoard::new(),
        _ => {
            panic!("Invalid piece type: {:?}", piece);
        }
    }
}

pub(crate) fn pseudo_legal_move_from(pos: &Position, sq: Square) -> BitBoard {
    pseudo_legal_from_impl::<false>(pos, sq, pos.side_to_move)
}

pub fn pseudo_legal_attack_from(pos: &Position, sq: Square, color: Color) -> BitBoard {
    pseudo_legal_from_impl::<true>(pos, sq, color)
}

/// Pseudo-legal move generation for pawns
fn move_pawn_enpassant<const COLOR: u8>(pos: &Position, sq: Square) -> BitBoard {
    // Handle en passant
    let (file, rank) = sq.file_rank();
    let is_white = COLOR == Color::WHITE.as_u8();
    let is_black = COLOR != Color::WHITE.as_u8();

    if let Some(ep_sq) = pos.ep_sq {
        debug_assert!(pos.get_piece(ep_sq) == Piece::NONE, "En passant square must be empty");
        let (ep_file, ep_rank) = ep_sq.file_rank();
        if (file as i32 - ep_file as i32).abs() == 1 {
            if is_white && rank == RANK_5 && ep_rank == RANK_6 {
                debug_assert!(pos.get_piece(Square(ep_sq.0 - 8)) == Piece::B_PAWN);
                return ep_sq.to_bitboard();
            }
            if is_black && rank == RANK_4 && ep_rank == RANK_3 {
                debug_assert!(pos.get_piece(Square(ep_sq.0 + 8)) == Piece::W_PAWN);
                return ep_sq.to_bitboard();
            }
        }
    }

    return BitBoard::new();
}

fn move_pawn<const COLOR: u8, const ATTACK_ONLY: bool>(pos: &Position, sq: Square) -> BitBoard {
    let (_file, rank) = sq.file_rank();
    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();

    let opponent = COLOR ^ 1;

    let is_white = COLOR == Color::WHITE.as_u8();
    let is_black = COLOR != Color::WHITE.as_u8();

    if !ATTACK_ONLY {
        // Handle forward moves
        let next_bb = if is_white { shift_north(bb) } else { shift_south(bb) };

        if (next_bb & pos.occupancies[Color::BOTH.as_usize()]).none() {
            moves |= next_bb;
        }

        if (is_white && rank == RANK_2 || is_black && rank == RANK_7) && moves.any() {
            let next_bb = if is_white { shift_north(next_bb) } else { shift_south(next_bb) };
            if (next_bb & pos.occupancies[Color::BOTH.as_usize()]).none() {
                moves |= next_bb;
            }
        }

        // Handle en passant
        moves |= move_pawn_enpassant::<COLOR>(pos, sq);
    }

    // Handle attacks moves
    let attack_left = if is_white { shift_nw(bb) } else { shift_sw(bb) };
    if ATTACK_ONLY || (attack_left & pos.occupancies[opponent as usize]).any() {
        moves |= attack_left;
    }
    let attack_right = if is_white { shift_ne(bb) } else { shift_se(bb) };
    if ATTACK_ONLY || (attack_right & pos.occupancies[opponent as usize]).any() {
        moves |= attack_right;
    }

    // @TODO: handle en passant - BEGIN
    // @TODO: handle en passant - END

    // @TODO: handle promotion - BEGIN
    {
        let mut promotion = false;
        match rank {
            RANK_7 if is_white => {
                promotion = true;
            }
            RANK_2 if is_black => {
                promotion = true;
            }
            _ => {}
        }
        if promotion {
            println!("TODO: handle promotion for pawn at square {}", sq);
        }
    }
    // @TODO: handle promotion - END

    moves
}

/// Pseudo-legal move generation for a sliding piece (rook, bishop, queen)
fn move_sliding<const START: u8, const END: u8>(
    pos: &Position,
    sq: Square,
    color: Color,
) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();
    let opponent = color.opponent();

    for i in START..END {
        let mut next_bb = SHIFT_FUNCS[i as usize](bb);

        while next_bb.any() {
            if (next_bb & pos.occupancies[color.as_usize()]).any() {
                break;
            }

            if (next_bb & pos.occupancies[opponent.as_usize()]).any() {
                moves |= next_bb;
                break;
            }

            moves |= next_bb;

            next_bb = SHIFT_FUNCS[i as usize](next_bb);
        }
    }

    moves
}

/// Pseudo-legal move generation for a knight
fn move_knight(pos: &Position, sq: Square, color: Color) -> BitBoard {
    let mut moves = BitBoard::new();
    let bb = sq.to_bitboard();
    let occupancy = !pos.occupancies[color.as_usize()];

    moves |= shift(bb & !(BOUND_AB | BOUND_1), SW + WEST) & occupancy;
    moves |= shift(bb & !(BOUND_AB | BOUND_8), NW + WEST) & occupancy;
    moves |= shift(bb & !(BOUND_GH | BOUND_1), SE + EAST) & occupancy;
    moves |= shift(bb & !(BOUND_GH | BOUND_8), NE + EAST) & occupancy;

    moves |= shift(bb & !(BOUND_A | BOUND_12), SW + SOUTH) & occupancy;
    moves |= shift(bb & !(BOUND_A | BOUND_78), NW + NORTH) & occupancy;
    moves |= shift(bb & !(BOUND_H | BOUND_12), SE + SOUTH) & occupancy;
    moves |= shift(bb & !(BOUND_H | BOUND_78), NE + NORTH) & occupancy;

    moves
}

/// Pseudo-legal move generation for a king
fn castling_check<const COLOR: u8>(
    pos: &Position,
    sq: Square,
    dst_sq: Square,
    rook_sq: Square,
) -> bool {
    // r . . . k . . r
    // a b c d e f g h
    let color = Color::from(COLOR);
    let opponent = color.opponent();

    // check if the rook is in the right place
    let rook_type = Piece::get_piece(color, PieceType::Rook);
    if pos.bitboards[rook_type.as_usize()].test(rook_sq.as_u8()) == false {
        return false;
    }

    fn min_max(a: Square, b: Square) -> (Square, Square) {
        if a.0 < b.0 { (a, b) } else { (b, a) }
    }

    // check if the cells are under attack
    let (start, end) = min_max(sq, dst_sq);
    for i in start.0..=end.0 {
        if pos.attack_map[opponent.as_usize()].test(i) {
            return false;
        }
    }

    // check if any piece is in the way
    let (start, end) = min_max(sq, rook_sq);
    for i in start.0 + 1..end.0 {
        if (pos.occupancies[Color::BOTH.as_usize()]).test(i) {
            return false;
        }
    }

    true
}

fn move_king<const COLOR: u8, const ATTACK_ONLY: bool>(pos: &Position, sq: Square) -> BitBoard {
    let color = Color::from(COLOR);
    let is_white = color.is_white();

    let bb = sq.to_bitboard();
    let mut moves = BitBoard::new();
    let occupancy = !pos.occupancies[COLOR as usize];
    moves |= shift_north(bb) & occupancy;
    moves |= shift_south(bb) & occupancy;
    moves |= shift_east(bb) & occupancy;
    moves |= shift_west(bb) & occupancy;
    moves |= shift_ne(bb) & occupancy;
    moves |= shift_nw(bb) & occupancy;
    moves |= shift_se(bb) & occupancy;
    moves |= shift_sw(bb) & occupancy;
    if !ATTACK_ONLY {
        // If we are checking if cells are being attacked, not actually moving, no need exclude pieces under attack
        moves &= !pos.attack_map[color.opponent().as_usize()];

        // Castling doesn't form attack
        // @TODO: refactor this part
        if is_white {
            if (pos.castling & MoveFlags::K != 0)
                && castling_check::<COLOR>(pos, sq, Square::G1, Square::H1)
            {
                assert!(sq == Square::E1);
                moves.set_sq(Square::G1);
            }
            if (pos.castling & MoveFlags::Q != 0)
                && castling_check::<COLOR>(pos, sq, Square::C1, Square::A1)
            {
                assert!(sq == Square::E1);
                moves.set_sq(Square::C1);
            }
        } else {
            if (pos.castling & MoveFlags::k != 0)
                && castling_check::<COLOR>(pos, sq, Square::G8, Square::H8)
            {
                assert!(sq == Square::E8);
                moves.set_sq(Square::G8);
            }
            if (pos.castling & MoveFlags::q != 0)
                && castling_check::<COLOR>(pos, sq, Square::C8, Square::A8)
            {
                assert!(sq == Square::E8);
                moves.set_sq(Square::C8);
            }
        }
    }

    moves
}

pub fn validate_move(pos: &mut Position, m: &Move) -> bool {
    let us = pos.side_to_move;
    let opponent = us.opponent();
    let piece: Piece = Piece::get_piece(us, PieceType::King);
    debug_assert!(piece == Piece::W_KING || piece == Piece::B_KING);
    debug_assert!(piece.color() == us);

    let snapshot = make_move(pos, m);

    let legal = (pos.bitboards[piece.as_usize()] & pos.attack_map[opponent.as_usize()]).none();

    unmake_move(pos, m, &snapshot);

    legal
}

/// Pseudo-legal move generation from a square to another
pub fn pseudo_legal_move_from_to(pos: &Position, from_sq: Square, to_sq: Square) -> Move {
    let mut from = Piece::NONE;
    let mut to = Piece::NONE;

    for i in 0..pos.bitboards.len() {
        let bb = &pos.bitboards[i];
        if bb.test(from_sq.as_u8()) {
            from = unsafe { std::mem::transmute(i as u8) };
        }
        if bb.test(to_sq.as_u8()) {
            to = unsafe { std::mem::transmute(i as u8) };
        }
    }

    debug_assert!(from != Piece::NONE, "No piece found on 'from' square");
    debug_assert!(
        from.color() == pos.side_to_move,
        "Piece on 'from' square is not of the current side"
    );
    debug_assert!(
        to.color() != from.color(),
        "Piece on 'to' square is of the same color as the piece on 'from' square"
    );

    let drop_ep_sq = pos.ep_sq;

    let add_ep_sq = check_if_add_eq_sq(pos, from_sq, to_sq, from);

    let is_ep_capture = check_if_is_eq_capture(pos, from_sq, to_sq, from, to);

    Move::new(from_sq, to_sq, from, to, drop_ep_sq, add_ep_sq, is_ep_capture)
}

fn check_if_is_eq_capture(
    pos: &Position,
    from_sq: Square,
    to_sq: Square,
    from: Piece,
    to: Piece,
) -> bool {
    if from.piece_type() != PieceType::Pawn {
        return false;
    }

    if to.piece_type() != PieceType::None {
        return false;
    }

    // 8 . . . . . k . . black pawn c7c5, c6 is empty, c5 has black pawn
    // 7 . . . . . . . .
    // 6 . . . . . . . .
    // 5 . . p P . . . .
    // 4 . . . . . . . .
    // 3 . . . . . . . .
    // 2 . . . . . . . .
    // 1 . . . . K . . .
    //   a b c d e f g h

    // if the to square is empty, but it still moves diagonally, then
    let (from_file, from_rank) = from_sq.file_rank();
    let (to_file, to_rank) = to_sq.file_rank();
    if from_file == to_file {
        return false;
    }
    debug_assert!((from_file as i8 - to_file as i8).abs() == 1);
    debug_assert!((from_rank as i8 - to_rank as i8).abs() == 1);

    if cfg!(debug_assertions) {
        let color = from.color();
        let enemy = if color == Color::WHITE { Piece::B_PAWN } else { Piece::W_PAWN };
        let enemy_sq = Square::make(to_file, from_rank);

        debug_assert!(
            pos.bitboards[enemy.as_usize()].test(enemy_sq.as_u8()),
            "En passant capture must have an enemy pawn on the square to capture"
        );
    }

    true
}

fn check_if_add_eq_sq(pos: &Position, from_sq: Square, to_sq: Square, from: Piece) -> bool {
    if from.piece_type() != PieceType::Pawn {
        return false;
    }

    let (file, from_rank) = from_sq.file_rank();
    let (_file, to_rank) = to_sq.file_rank();

    if match (from, from_rank, to_rank) {
        (Piece::W_PAWN, RANK_2, RANK_4) => true,
        (Piece::B_PAWN, RANK_7, RANK_5) => true,
        _ => false,
    } {
        assert_eq!(file, _file);
        // check if there's opponent's pawn on the left or right of 'to' square
        let board = &pos.bitboards[if from == Piece::W_PAWN {
            Piece::B_PAWN.as_usize()
        } else {
            Piece::W_PAWN.as_usize()
        }];

        if file < FILE_H && board.test(Square::make(file + 1, to_rank).as_u8()) {
            return true;
        }
        if file > FILE_A && board.test(Square::make(file - 1, to_rank).as_u8()) {
            return true;
        }
    }

    false
}

pub fn legal_move_from_to(pos: &mut Position, from_sq: Square, to_sq: Square) -> Option<Move> {
    // @TODO: this is not technically a legal move check
    if !pseudo_legal_move_from(pos, from_sq).test(to_sq.as_u8()) {
        return None;
    }

    Some(pseudo_legal_move_from_to(pos, from_sq, to_sq))
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

pub fn to_string(pos: &Position, pad: bool) -> String {
    let mut s = String::new();
    for rank in (0..8).rev() {
        s.push((rank as u8 + b'1') as char);
        s.push(' ');
        for file in 0..8 {
            let sq = rank * 8 + file;
            let piece_char = if pos.bitboards[Piece::W_PAWN.as_usize()].test(sq) {
                '♙'
            } else if pos.bitboards[Piece::W_KNIGHT.as_usize()].test(sq) {
                '♘'
            } else if pos.bitboards[Piece::W_BISHOP.as_usize()].test(sq) {
                '♗'
            } else if pos.bitboards[Piece::W_ROOK.as_usize()].test(sq) {
                '♖'
            } else if pos.bitboards[Piece::W_QUEEN.as_usize()].test(sq) {
                '♕'
            } else if pos.bitboards[Piece::W_KING.as_usize()].test(sq) {
                '♔'
            } else if pos.bitboards[Piece::B_PAWN.as_usize()].test(sq) {
                '♟'
            } else if pos.bitboards[Piece::B_KNIGHT.as_usize()].test(sq) {
                '♞'
            } else if pos.bitboards[Piece::B_BISHOP.as_usize()].test(sq) {
                '♝'
            } else if pos.bitboards[Piece::B_ROOK.as_usize()].test(sq) {
                '♜'
            } else if pos.bitboards[Piece::B_QUEEN.as_usize()].test(sq) {
                '♛'
            } else if pos.bitboards[Piece::B_KING.as_usize()].test(sq) {
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
    s.push_str(format!("Side: {}\n", pos.side_to_move).as_str());
    s.push_str(format!("Castling: {}\n", castling_to_string(pos.castling)).as_str());
    match pos.ep_sq {
        Some(ep_sq) => s.push_str(format!("En passant: {}\n", ep_sq).as_str()),
        None => s.push_str("En passant: -\n"),
    }
    s.push_str(format!("Halfmove clock: {}\n", pos.halfmove_clock).as_str());
    s.push_str(format!("Fullmove number: {}\n", pos.fullmove_number).as_str());

    s
}

pub fn to_board_string(pos: &Position) -> String {
    let mut s = String::new();
    for rank in (0..8).rev() {
        for file in 0..8 {
            let sq = rank * 8 + file;
            let piece = pos.get_piece(Square(sq));
            s.push(piece.to_char());
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn squares_to_bitboard(sqs: &[Square]) -> BitBoard {
        let mut bb = BitBoard::new();
        for &sq in sqs {
            bb.set_sq(sq);
        }
        bb
    }

    const BB_D4: BitBoard = Square::D4.to_bitboard();
    const BB_D5: BitBoard = Square::D5.to_bitboard();
    const BB_D6: BitBoard = Square::D6.to_bitboard();

    const BB_E3: BitBoard = Square::E3.to_bitboard();
    const BB_E4: BitBoard = Square::E4.to_bitboard();
    const BB_E5: BitBoard = Square::E5.to_bitboard();

    #[test]
    fn test_white_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();
        assert_eq!(
            pos.to_board_string(),
            "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR"
        );

        let moves = pseudo_legal_move_from(&pos, Square::E2);
        assert_eq!(moves, BB_E3 | BB_E4);
    }

    #[test]
    fn test_black_pawn_pseudo_legal_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::D7);
        assert_eq!(moves, BB_D6 | BB_D5);
    }

    #[test]
    fn white_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 w - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E4);
        assert_eq!(moves, BB_D5 | BB_E5);
    }

    #[test]
    fn black_pawn_attack() {
        let fen = "r2q1rk1/pp2bppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::D5);
        assert_eq!(moves, BB_D4 | BB_E4);
    }

    #[test]
    fn move_biship() {
        let fen = "rn1qkbnr/ppp1pppp/4b3/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E6);
        assert_eq!(
            moves,
            squares_to_bitboard(&[Square::C8, Square::D7, Square::F5, Square::G4, Square::H3])
        );
    }

    #[test]
    fn move_rook() {
        let fen = "rnbqkbn1/ppp1pp1r/7p/3p2p1/7P/3P1PPR/PPP1P3/RNBQKBN1 b - - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::H7);
        assert_eq!(moves, squares_to_bitboard(&[Square::H8, Square::G7]));
    }

    #[test]
    fn move_knight() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::B1);
        assert_eq!(moves, squares_to_bitboard(&[Square::A3, Square::C3]));
    }

    #[test]
    fn knight_attack() {
        let fen = "rn2kb1r/pppqppp1/5n2/3p3p/4P1b1/BPN5/P1PP1PPP/R2QKBNR b KQkq - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::F6);
        assert_eq!(moves, squares_to_bitboard(&[Square::E4, Square::G8, Square::H7]));
    }

    #[test]
    fn test_castling() {
        let fen = "r3k2r/ppp1bppp/2n1pn2/3p4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b kq - 0 10";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E8);
        assert_eq!(
            moves,
            squares_to_bitboard(&[Square::C8, Square::D8, Square::F8, Square::G8, Square::D7,])
        );
    }

    #[test]
    fn test_castling_2() {
        let fen = "8/4k3/8/8/8/8/r6R/R3K3 w Q - 0 1";
        let pos = Position::from(fen).unwrap();

        let moves = pseudo_legal_move_from(&pos, Square::E1);
        assert!(moves.test(Square::C1.as_u8()))
    }

    #[test]
    fn test_move_validation() {
        // 2 . . . . . . . k
        // 1 K B . . . . . r
        //   a b c d e f g h
        let mut pos = Position::from("8/8/8/8/8/8/7k/KB5r w - - 0 1").unwrap();

        assert_eq!(
            pos.attack_map[Color::BLACK.as_usize()],
            BitBoard::from(0b11000000_01000000_01111110)
        );

        let m = pseudo_legal_move_from_to(&pos, Square::B1, Square::A2);

        assert!(!validate_move(&mut pos, &m), "Move bishop to A2 exposes king to check");
    }

    #[test]
    fn test_en_passant() {
        let mut pos = Position::from("4k3/8/8/4Pp2/8/8/8/4K3 w - f6 2 4").unwrap();

        // 8 . . . . k . . .
        // 7 . . . . . . . .
        // 6 . . . . . . . .
        // 5 . . . . P p . .
        // 4 . . . . . . . .
        // 3 . . . . . . . .
        // 2 . . . . . . . .
        // 1 . . . . K . . .
        //   a b c d e f g h

        assert_eq!(pos.side_to_move, Color::WHITE);
        assert_eq!(pos.castling, 0);
        assert_eq!(pos.ep_sq.unwrap(), Square::F6);
        assert_eq!(pos.halfmove_clock, 2);
        assert_eq!(pos.fullmove_number, 4);
        assert_eq!(
            pos.to_board_string(),
            "....k.......................Pp..............................K..."
        );

        let legal_moves = pos.legal_move(Square::E5);
        assert_eq!(legal_moves, Square::E6.to_bitboard() | Square::F6.to_bitboard());
    }
}

// @TODO: get rid of these constants
