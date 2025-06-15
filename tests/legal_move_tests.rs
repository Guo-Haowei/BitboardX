use bitboard_x::engine::board::*;
use bitboard_x::engine::move_gen::*;
use bitboard_x::engine::position::*;
use bitboard_x::engine::types::*;

#[test]
fn test_pin() {
    // 2 . . . . . . . k
    // 1 K B . . . . . r
    //   a b c d e f g h
    let pos = Position::from("8/8/8/8/8/8/7k/KB5r w - - 0 1").unwrap();

    let is_pinned = pos.is_square_pinned(Square::B1, Color::WHITE);

    assert!(is_pinned, "Move bishop to A2 exposes king to check");
}

#[test]
fn test_rook_pin_pawn() {
    let pos = Position::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

    let is_pinned = pos.is_square_pinned(Square::B5, Color::WHITE);

    assert!(is_pinned, "Pawn B5 is pinned by rook on H5");
}

#[test]
fn capture_resolve_check() {
    let mut pos =
        Position::from("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .unwrap();

    let m = Move::new(Square::B2, Square::B3, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m));
    pos.make_move(m);

    let m = Move::new(Square::C5, Square::E3, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m));
    pos.make_move(m);

    let m = Move::new(Square::F2, Square::E3, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m), "Capture on E3 should resolve check");
}

#[test]
fn en_passant_expose_check() {
    let mut pos = Position::from("8/8/8/KP5r/1R3p1k/8/4P3/8 w - - 0 1").unwrap();
    let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&pos, &m), "E2 to E4 should be legal");
    pos.make_move(m);

    let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
    assert!(
        !is_pseudo_move_legal(&pos, &m),
        "En passant is illegal because it exposes king to check"
    );
}

#[test]
fn en_passant_capture_checker() {
    let mut pos = Position::from("8/8/8/KP1k3r/1R3p2/8/4P3/8 w - - 0 1").unwrap();
    let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&mut pos, &m), "E2 to E4 should be legal");
    pos.make_move(m);

    let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
    assert!(
        is_pseudo_move_legal(&mut pos, &m),
        "En passant is illegal it captures the attacking pawn"
    );
}

#[test]
fn move_king_along_the_checker_line() {
    let mut pos =
        Position::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    let m = Move::new(Square::E1, Square::F2, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&mut pos, &m));
    pos.make_move(m);

    let m = Move::new(Square::E7, Square::H4, MoveType::Normal, None);
    assert!(is_pseudo_move_legal(&mut pos, &m),);
    pos.make_move(m);
    let m = Move::new(Square::F2, Square::E1, MoveType::Normal, None);
    assert!(
        !is_pseudo_move_legal(&mut pos, &m),
        "King cannot move along the line of the checking piece, in this case from F2 to E1"
    );
    pos.make_move(m);
}

#[test]
fn move_king_along_the_checker_line_2() {
    let pos = Position::from("8/2p5/3p4/KP6/5pk1/7P/4P3/6R1 b - - 0 1").unwrap();

    let moves = legal_moves(&pos);

    assert_eq!(moves.count(), 4, "There should be 4 legal moves");
}
