use bitboard_x::ai::*;
use bitboard_x::core::position::*;
use bitboard_x::named_test;

use colored::*;

named_test!(find_mate_in_two, {
    const SEARCH_DEPTH: u8 = 3;
    let fen = "r4r1k/2p1p2p/p5p1/1p1Q1p2/1P3bq1/P1P2N2/1B3P2/4R1RK b - - 0 1";
    let mut pos = Position::from_fen(fen).unwrap();
    let mv = search(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "g4h3");
    pos.make_move(mv);

    let mv = search(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "f3h2");
    pos.make_move(mv);

    let mv = search(&mut pos, SEARCH_DEPTH).unwrap();
    assert_eq!(mv.to_string(), "h3h2");
    pos.make_move(mv);
});
