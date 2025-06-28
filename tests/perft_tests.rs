/// Test cases: https://www.chessprogramming.org/Perft_Results
///
use colored::*;
use pretty_assertions::assert_eq;
use std::time::Instant;

use bitboard_x::named_test;

const DEFAULT_DEPTH: u8 = if cfg!(not(debug_assertions)) { 8 } else { 5 };

fn perft_test_wrapper(fen: &str, depth: u8, expectations: &Vec<u64>) {
    let engine = bitboard_x::engine::Engine::from_fen(fen).unwrap();

    let mut out = std::io::sink();

    for (i, expected) in expectations.iter().enumerate() {
        let test_depth = i as u8;
        if test_depth > depth {
            break; // Skip depth 0, which is always 1
        }

        let now = Instant::now(); // Start timer
        let actual = engine.perft_test(&mut out, test_depth);
        let elapsed = now.elapsed();
        let msg = format!("depth {}: {} nodes, took {:?}", test_depth, actual, elapsed);
        println!("{}", if actual == *expected { msg.green() } else { msg.red() });
        assert_eq!(actual, *expected);
    }
}

named_test!(perft_initial_position, {
    let tests = vec![
        1u64,
        20u64,
        400u64,
        8902u64,
        197281u64,
        4865609u64,
        119060324u64,
        3195901860u64, // depth 7
                       // 84998978956u64, // depth 8
    ];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", depth, &tests);
});

named_test!(perft_test_position2, {
    let tests = vec![1u64, 48u64, 2039u64, 97862u64, 4085603u64, 193690690u64, 8031647685u64];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        depth,
        &tests,
    );
});

named_test!(test_position3, {
    let tests = vec![
        1u64,
        14u64,
        191u64,
        2812u64,
        43238u64,
        674624u64,
        11030083u64,
        178633661u64,  // depth 7
        3009794393u64, // depth 8
    ];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", depth, &tests);
});

named_test!(test_position4, {
    let tests = vec![1, 6, 264, 9467, 422333, 15833292, 706045033];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        depth,
        &tests,
    );
});

named_test!(test_position5, {
    let tests = vec![1u64, 44u64, 1486u64, 62379u64, 2103487u64, 89941194u64];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", depth, &tests);
});

named_test!(test_position6, {
    let tests = vec![
        1u64,
        46u64,
        2079u64,
        89890u64,
        3894594u64,
        164075551u64,
        6923051137u64,
        287188994746u64,
        11923589843526u64, // depth 8
    ];

    // depth 7 takes too long
    let depth = DEFAULT_DEPTH.min(6);
    perft_test_wrapper(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        depth,
        &tests,
    );
});
