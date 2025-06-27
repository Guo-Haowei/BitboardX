use bitboard_x::engine::Engine;
use bitboard_x::named_test;

use colored::*;

const SEARCH_DEPTH: u8 = 4;

named_test!(find_mate_in_two, {
    let fen = "r4r1k/2p1p2p/p5p1/1p1Q1p2/1P3bq1/P1P2N2/1B3P2/4R1RK b - - 0 1";
    let mut engine = Engine::from_fen(fen).unwrap();
    let mv = engine.best_move_depth(SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "g4h3");
    engine.make_move("g4h3");

    let mv = engine.best_move_depth(SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "f3h2");
    engine.make_move("f3h2");

    let mv = engine.best_move_depth(SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "h3h2");
    engine.make_move("h3h2");
});

named_test!(find_mate_in_three, {
    let fen = "Q4bk1/p2b1r2/7p/1pp5/4P1pq/2NP2P1/PPn3P1/1RB2RK1 b - - 0 1";
    let mut engine = Engine::from_fen(fen).unwrap();
    let mv = engine.best_move_depth(SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "f7f1");
});

named_test!(should_capture_queen, {
    let fen = "r1b1kb1r/1p1n1ppp/p2p4/8/5P2/4n1N1/PPP3PP/R1K2Q1R b kq - 1 3";
    let mut engine = Engine::from_fen(fen).unwrap();
    let mv = engine.best_move_depth(SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "e3f1");
});

named_test!(should_avoid_checkmate, {
    let fen = "3r2k1/1p3p1p/6p1/8/5n2/1R1b1P2/PP1P1b1P/R1BK4 w - - 0 1";
    let mut engine = Engine::from_fen(fen).unwrap();
    let mv = engine.best_move_depth(6).unwrap();
    assert_ne!(mv.to_string(), "b2b7");
});
// iterative deepening
