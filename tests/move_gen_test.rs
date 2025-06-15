use bitboard_x::engine::position::*;

/// Test cases: https://www.chessprogramming.org/Perft_Results
use colored::*;
use pretty_assertions::assert_eq;
use std::time::Instant;

const DEFAULT_DEPTH: u8 = if cfg!(not(debug_assertions)) { 5 } else { 4 };

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
        let snapshot = pos.make_move(m.clone());
        nodes += perft_test(pos, depth - 1);
        pos.unmake_move(m.clone(), &snapshot);
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
        println!("{}", if actual == expected { msg.green() } else { msg.red() });
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

    perft_test_wrapper(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        DEFAULT_DEPTH,
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
        (0, 1u64),
        (0, 1u64),
    ];

    perft_test_wrapper(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        DEFAULT_DEPTH,
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

    perft_test_wrapper("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", DEFAULT_DEPTH, &tests);
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
        (0, 1u64),
        (0, 1u64),
    ];

    perft_test_wrapper(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        DEFAULT_DEPTH,
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
        (0, 1u64), // No known results for depth 6
        (0, 1u64), // No known results for depth 7
        (0, 1u64), // No known results for depth 8
    ];

    perft_test_wrapper(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        DEFAULT_DEPTH,
        &tests,
    );
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

    perft_test_wrapper(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        DEFAULT_DEPTH,
        &tests,
    );
}
