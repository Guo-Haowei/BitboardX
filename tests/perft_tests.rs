/// Test cases: https://www.chessprogramming.org/Perft_Results
///
use colored::*;
use pretty_assertions::assert_eq;
use std::thread;
use std::time::Instant;

use bitboard_x::engine::{move_gen, position::*};
use bitboard_x::named_test;

const DEFAULT_DEPTH: u8 = if cfg!(not(debug_assertions)) { 8 } else { 5 };

fn perft_test_inner(pos: &mut Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let move_list = move_gen::legal_moves(pos);

    if depth == 1 {
        return move_list.len() as u64;
    }

    let mut nodes = 0u64;
    for m in move_list.iter() {
        let snapshot = pos.make_move(m.clone());
        nodes += perft_test_inner(pos, depth - 1);
        pos.unmake_move(m.clone(), &snapshot);
    }

    nodes
}

fn perft_test(pos: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let move_list = move_gen::legal_moves(pos);
    let move_count = move_list.len() as u64;

    if depth == 1 {
        return move_count;
    }

    let mut handles = Vec::new();

    for mv in move_list.iter() {
        let mut child = pos.clone();
        child.make_move(mv.clone());
        let handle = thread::spawn(move || perft_test_inner(&mut child, depth - 1));
        handles.push(handle);
    }

    // Wait for all threads and sum the results
    let mut nodes = 0;
    for handle in handles {
        nodes += handle.join().expect("Thread panicked");
    }

    nodes
}

fn perft_test_wrapper(fen: &str, depth: u8, expectations: &Vec<u64>) {
    let mut pos = Position::from(fen).unwrap();

    for (i, expected) in expectations.iter().enumerate() {
        let test_depth = i as u8;
        if test_depth > depth {
            break; // Skip depth 0, which is always 1
        }

        let start = Instant::now(); // Start timer
        let actual = perft_test(&mut pos, test_depth);
        let elapsed = start.elapsed();
        let msg = format!("depth {}: {} nodes, took {:?}", test_depth, actual, elapsed);
        println!("{}", if actual == *expected { msg.green() } else { msg.red() });
        assert_eq!(actual, *expected);
    }
}

named_test!(perft_initial_position, {
    let tests = vec![
        1, 20, 400, 8902, 197281, 4865609, 119060324,
        3195901860, // depth 7
                   // 84998978956, //
    ];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", depth, &tests);
});

named_test!(perft_test_position2, {
    let tests = vec![1, 48, 2039, 97862, 4085603, 193690690, 8031647685];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        depth,
        &tests,
    );
});

named_test!(test_position3, {
    let tests = vec![
        1,
        14,
        191,
        2812,
        43238,
        674624,
        11030083,
        178633661, // depth 7
        3009794393u64,
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
    let tests = vec![1, 44, 1486, 62379, 2103487, 89941194];

    let depth = DEFAULT_DEPTH.min(tests.len() as u8 - 1);
    perft_test_wrapper("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", depth, &tests);
});

named_test!(test_position6, {
    let tests = vec![
        1, 46, 2079, 89890, 3894594, 164075551,
        6923051137,
        // 287188994746,
        // 11923589843526u64,
    ];

    // depth 7 takes too long
    let depth = DEFAULT_DEPTH.min(6);
    perft_test_wrapper(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        depth,
        &tests,
    );
});
