use bitboard_x::engine::board::*;
use bitboard_x::engine::move_gen::is_pseudo_move_legal;
use bitboard_x::engine::position::Position;
use bitboard_x::named_test;

use colored::*;

named_test!(king_capture_resolves_check_is_legal, {
    let mut pos =
        Position::from("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 b - - 0 10")
            .unwrap();

    let m = Move::new(Square::C5, Square::E3, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m));
    pos.make_move(m);

    let m = Move::new(Square::F2, Square::E3, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m), "Capture on E3 should resolve check");
});

named_test!(en_passant_expose_check_is_illegal, {
    let mut pos = Position::from("8/8/8/KP5r/1R3p1k/8/4P3/8 w - - 0 1").unwrap();
    let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m), "E2 to E4 should be legal");
    pos.make_move(m);

    let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
    assert!(!is_pseudo_move_legal(&pos, &m));
});

named_test!(en_passant_capture_checker_is_legal, {
    let mut pos = Position::from("8/8/8/KP1k3r/1R3p2/8/4P3/8 w - - 0 1").unwrap();
    let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m), "E2 to E4 should be legal");
    pos.make_move(m);

    let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
    assert!(is_pseudo_move_legal(&pos, &m));
});

named_test!(move_king_along_the_checker_line, {
    let mut pos =
        Position::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    let m = Move::new(Square::E1, Square::F2, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m));
    pos.make_move(m);

    let m = Move::new(Square::E7, Square::H4, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m),);
    pos.make_move(m);
    let m = Move::new(Square::F2, Square::E1, MoveType::Normal, None);
    assert!(!is_pseudo_move_legal(&pos, &m));
    pos.make_move(m);
});

named_test!(move_king_along_the_checker_line_2, {
    let pos = Position::from("8/2p5/3p4/KP6/5pk1/7P/4P3/6R1 b - - 0 1").unwrap();

    let m = Move::new(Square::G4, Square::G5, MoveType::Normal, None);
    assert!(!is_pseudo_move_legal(&pos, &m));
});
