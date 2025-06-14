use crate::engine::move_gen::internal::pseudo_legal_move_from_to;

use super::board::*;
use super::position::Position;
use super::types::*;

mod internal;

/// Pseudo-legal move generation
pub fn pseudo_legal_moves(pos: &Position) -> MoveList {
    let mut moves = MoveList::new();

    let color = pos.side_to_move;
    let (start, end) = if color == Color::WHITE {
        (Piece::W_START, Piece::W_END)
    } else {
        (Piece::B_START, Piece::B_END)
    };

    for i in start..=end {
        let mut bb = pos.bitboards[i as usize];
        while bb.any() {
            let sq = bb.first_nonzero_sq();

            let mut bb2 = internal::pseudo_legal_move_from(pos, sq);

            while bb2.any() {
                let m = internal::pseudo_legal_move_from_to(pos, sq, bb2.first_nonzero_sq());
                moves.add(m);
                bb2.remove_first_nonzero_sq();
            }

            bb.remove_first_nonzero_sq();
        }
    }

    moves
}

/// Legal move generation
pub fn legal_moves(pos: &mut Position) -> MoveList {
    let pseudo_moves = pseudo_legal_moves(pos);
    let mut moves = MoveList::new();
    for m in pseudo_moves.iter() {
        if internal::is_move_legal(pos, m) {
            moves.add(m.clone());
        }
    }

    moves
}

pub fn is_move_legal(pos: &mut Position, m: &Move) -> bool {
    internal::is_move_legal(pos, m)
}

pub fn calc_attack_map_impl<const COLOR: u8, const START: u8, const END: u8>(
    pos: &Position,
) -> BitBoard {
    let mut attack_map = BitBoard::new();

    for i in START..=END {
        // pieces from W to B
        let bb = pos.bitboards[i as usize];
        for sq in 0..64 {
            if bb.test(sq) {
                attack_map |=
                    internal::pseudo_legal_attack_from(pos, Square(sq), Color::from(COLOR));
            }
        }
    }

    attack_map
}

#[cfg(test)]
/// Test cases: https://www.chessprogramming.org/Perft_Results
mod perft {
    use colored::*;
    use pretty_assertions::assert_eq;
    use std::time::Instant;

    use crate::engine::position::Position;

    fn perft_test(pos: &mut Position, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let move_list = pos.legal_moves();

        if depth == 1 {
            return move_list.count() as u64;
        }

        let mut nodes = 0u64;
        for m in move_list.iter() {
            let snapshot = pos.make_move(&m);
            nodes += perft_test(pos, depth - 1);
            pos.unmake_move(&m, &snapshot);
        }

        nodes
    }

    fn perft_test_wrapper(fen: &str, depth: u8, tests: &[(u8, u64); 9]) {
        let mut pos = Position::from(fen).unwrap();
        assert!(depth < 8);

        for i in 0..=depth {
            let (test_depth, expected) = tests[i as usize];

            let start = Instant::now(); // Start timer
            let actual = perft_test(&mut pos, test_depth);
            let elapsed = start.elapsed();
            let msg = format!("\nDepth {}: {} nodes, took {:?}", test_depth, actual, elapsed);
            println!("{}", msg.green());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_initial_position() {
        let tests = [
            (0, 1),
            (1, 20),
            (2, 400),
            (3, 8902),
            (4, 197281),
            (5, 4865609),
            (6, 119060324),
            (7, 3195901860),
            (8, 84998978956),
        ];

        const DEPTH: u8 = if cfg!(not(debug_assertions)) { 4 } else { 3 };
        perft_test_wrapper(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            DEPTH,
            &tests,
        );
    }

    #[test]
    fn test_position2() {
        let tests = [
            (0, 1),
            (1, 48),
            (2, 2039),
            (3, 97862),
            (4, 4085603),
            (5, 193690690),
            (6, 8031647685),
            (7, 0u64),
            (8, 0u64),
        ];

        const DEPTH: u8 = if cfg!(not(debug_assertions)) { 3 } else { 3 };
        perft_test_wrapper(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            DEPTH,
            &tests,
        );
    }

    #[test]
    fn test_position3() {
        let tests = [
            (0, 1),
            (1, 14),
            (2, 191),
            (3, 2812),
            (4, 43238),
            (5, 674624),
            (6, 11030083),
            (7, 178633661),
            (8, 3009794393u64),
        ];

        const DEPTH: u8 = if cfg!(not(debug_assertions)) { 4 } else { 3 };
        perft_test_wrapper("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ", DEPTH, &tests);
    }

    #[test]
    fn test_position4() {
        let tests = [
            (0, 1),
            (1, 6),
            (2, 264),
            (3, 9467),
            (4, 422333),
            (5, 15833292),
            (6, 706045033),
            (7, 0u64),
            (8, 0u64),
        ];

        perft_test_wrapper(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
            1,
            &tests,
        );
    }

    #[test]
    fn test_position5() {
        let tests = [
            (0, 1),
            (1, 44),
            (2, 1486),
            (3, 62379),
            (4, 2103487),
            (5, 89941194),
            (6, 0u64), // No known results for depth 6
            (7, 0u64), // No known results for depth 7
            (8, 0u64), // No known results for depth 8
        ];

        perft_test_wrapper("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 0, &tests);
    }

    #[test]
    fn test_position6() {
        let tests = [
            (0, 1),
            (1, 46),
            (2, 2079),
            (3, 89890),
            (4, 3894594),
            (5, 164075551),
            (6, 6923051137),
            (7, 287188994746),
            (8, 11923589843526u64),
        ];

        const DEPTH: u8 = if cfg!(not(debug_assertions)) { 4 } else { 3 };

        perft_test_wrapper(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            DEPTH,
            &tests,
        );
    }
}
