use bitboard_x::ai::*;
use bitboard_x::core::position::*;
use bitboard_x::named_test;

use colored::*;

named_test!(find_mate_in_two, {
    const SEARCH_DEPTH: u8 = 3;
    let fen = "r4r1k/2p1p2p/p5p1/1p1Q1p2/1P3bq1/P1P2N2/1B3P2/4R1RK b - - 0 1";
    let mut pos = Position::from_fen(fen).unwrap();
    let mv = find_best_move(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "g4h3");
    pos.make_move(mv);

    let mv = find_best_move(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "f3h2");
    pos.make_move(mv);

    let mv = find_best_move(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "h3h2");
    pos.make_move(mv);
});

named_test!(find_mate_in_three, {
    const SEARCH_DEPTH: u8 = 3;
    let fen = "Q4bk1/p2b1r2/7p/1pp5/4P1pq/2NP2P1/PPn3P1/1RB2RK1 b - - 0 1";
    let mut pos = Position::from_fen(fen).unwrap();
    let mv = find_best_move(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "f7f1");
    pos.make_move(mv);
});
