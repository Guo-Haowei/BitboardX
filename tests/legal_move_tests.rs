use bitboard_x::engine::position::Position;
use bitboard_x::engine::{board::*, move_gen};
use bitboard_x::named_test;

use colored::*;

fn is_move_legal(pos: &Position, m: Move) -> bool {
    let pseudo_moves = move_gen::legal_moves(pos);
    for lm in pseudo_moves.iter() {
        if m.equals(lm) {
            return true;
        }
    }

    false
}

named_test!(king_capture_resolves_check_is_legal, {
    let mut pos =
        Position::from("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 b - - 0 10")
            .unwrap();

    let m = Move::new(Square::C5, Square::E3, MoveType::Normal, None);
    assert!(is_move_legal(&pos, m));
    pos.make_move(m);

    let m = Move::new(Square::F2, Square::E3, MoveType::Normal, None);
    assert!(is_move_legal(&pos, m), "Capture on E3 should resolve check");
});

named_test!(move_king_along_bishop_checker_line, {
    let mut pos =
        Position::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    let m = Move::new(Square::E1, Square::F2, MoveType::Normal, None);
    assert!(is_move_legal(&pos, m));
    pos.make_move(m);

    let m = Move::new(Square::E7, Square::H4, MoveType::Normal, None);
    assert!(is_move_legal(&pos, m),);
    pos.make_move(m);
    let m = Move::new(Square::F2, Square::E1, MoveType::Normal, None);
    assert!(!is_move_legal(&pos, m));
    pos.make_move(m);
});

named_test!(move_king_along_rook_checker_line, {
    let pos = Position::from("8/2p5/3p4/KP6/5pk1/7P/4P3/6R1 b - - 0 1").unwrap();

    let m = Move::new(Square::G4, Square::G5, MoveType::Normal, None);
    assert!(!is_move_legal(&pos, m));
});

named_test!(move_king_along_the_checker_line_2, {
    let pos = Position::from("8/2p5/3p4/KP6/5pk1/7P/4P3/6R1 b - - 0 1").unwrap();

    let m = Move::new(Square::G4, Square::G5, MoveType::Normal, None);
    assert!(!is_move_legal(&pos, m));
});

named_test!(move_other_another_piece_while_king_is_in_check, {
    let pos = Position::from("8/2p5/3p4/KP6/5pk1/7P/4P3/6R1 b - - 0 1").unwrap();

    let m = Move::new(Square::D6, Square::D5, MoveType::Normal, None);
    assert!(!is_move_legal(&pos, m));
});

mod basic_movement {
    use super::*;

    const POSITION_1: &str = "r2q1rk1/pp3ppp/2n1pn2/2bp4/4P3/2NP1N2/PPQ2PPP/R1B2RK1 b - - 0 10";

    named_test!(pawn_movement, {
        let pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::D5, Square::D4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m), "Black pawn can move forward one square");
        let m = Move::new(Square::D5, Square::E4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m), "Black pawn can capture white pawn");
        let m = Move::new(Square::D5, Square::C4, MoveType::Normal, None);
        assert!(!is_move_legal(&pos, m), "Black pawn can't capture empty square");
    });

    named_test!(bishop_movement, {
        let pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::C5, Square::F2, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m), "Black bishop can capture white pawn");
        let m = Move::new(Square::C5, Square::G1, MoveType::Normal, None);
        assert!(!is_move_legal(&pos, m), "Black bishop can't capture piece behind pin");
    });

    named_test!(knight_movement, {
        let pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::F6, Square::E4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m), "Black knight can capture");
        let m = Move::new(Square::F6, Square::G4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m), "Black knight can move to empty square");
        let m = Move::new(Square::F6, Square::D5, MoveType::Normal, None);
        assert!(!is_move_legal(&pos, m), "Black knight can't move to square occupied by own piece");
    });
}

mod castling {
    use super::*;

    const POSITION_1: &str = "r3k2r/pppbqppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPPBQPPP/R3K2R w KQkq - 0 1";

    named_test!(white_king_can_castle_kingside, {
        let pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::E1, Square::G1, MoveType::Castling, None);
        assert!(is_move_legal(&pos, m));
    });

    named_test!(white_king_can_castle_queenside, {
        let pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::E1, Square::C1, MoveType::Castling, None);
        assert!(is_move_legal(&pos, m));
    });

    const POSITION_2: &str = "r3k2r/pppbqppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPPBQPPP/R3K2R b KQkq - 0 1";

    named_test!(black_king_can_castle_kingside, {
        let pos = Position::from(POSITION_2).unwrap();
        let m = Move::new(Square::E8, Square::G8, MoveType::Castling, None);
        assert!(is_move_legal(&pos, m));
    });

    named_test!(black_king_can_castle_queenside, {
        let pos = Position::from(POSITION_2).unwrap();
        let m = Move::new(Square::E8, Square::C8, MoveType::Castling, None);
        assert!(is_move_legal(&pos, m));
    });

    const POSITION_3: &str = "r2bk2r/8/4B3/8/8/4b3/8/R3K2R w KQkq - 0 1";

    named_test!(white_king_cant_castle_kingside_g1_under_attack, {
        let pos = Position::from(POSITION_3).unwrap();
        let m = Move::new(Square::E1, Square::G1, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(white_king_cant_castle_queenside_c1_under_attack, {
        let pos = Position::from(POSITION_3).unwrap();
        let m = Move::new(Square::E1, Square::C1, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m));
    });

    const POSITION_4: &str = "r2nk2r/8/4B3/8/8/4b3/8/R3K2R b KQkq - 0 1";

    named_test!(black_king_cant_castle_kingside_g8_under_attack, {
        let pos = Position::from(POSITION_4).unwrap();
        let m = Move::new(Square::E8, Square::G8, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(black_king_cant_castle_queenside_d8_blocked, {
        let pos = Position::from(POSITION_4).unwrap();
        let m = Move::new(Square::E8, Square::C8, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(white_king_cant_castle_because_it_is_under_attack, {
        let pos = Position::from("r2bk2r/8/4B3/8/8/8/3b4/R3K2R w KQkq - 0 1").unwrap();
        let m = Move::new(Square::E1, Square::C1, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(white_king_cant_castle_because_rook_was_taken_out, {
        let mut pos = Position::from("r2bk2r/8/4B3/8/4b3/8/8/R3K2R b KQkq - 0 1").unwrap();
        let m = Move::new(Square::E4, Square::H1, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);
        let m = Move::new(Square::E1, Square::G1, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(castling_should_follow_fen_rules, {
        let mut pos = Position::from("r3k2r/8/8/8/4b3/8/8/R3K2R w Kq - 0 1").unwrap();
        let m = Move::new(Square::E1, Square::C1, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m), "white can't no castle queenside");

        let m = Move::new(Square::E1, Square::G1, MoveType::Castling, None);
        assert!(is_move_legal(&pos, m), "white can castle kingside");

        pos.make_move(m);

        let m = Move::new(Square::E8, Square::C8, MoveType::Castling, None);
        assert!(is_move_legal(&pos, m), "black can castle queenside");

        let m = Move::new(Square::E8, Square::G8, MoveType::Castling, None);
        assert!(!is_move_legal(&pos, m), "black can't castle kingside");
    });
}

mod en_passant {
    use super::*;

    // 8 k . . . . . . .
    // 7 . . . . . . . .
    // 6 . . . . . . . .
    // 5 . p P . . . . .
    // 4 . . . . . p . .
    // 3 . . . . . . . .
    // 2 . . . . P . . .
    // 1 . . . . . . . K
    //   a b c d e f g h

    const POSITION_1: &str = "k7/8/8/1pP5/5p2/8/4P3/7K w - b6 0 1";

    named_test!(white_pawn_can_capture_black_pawn_at_b6, {
        let pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::C5, Square::B6, MoveType::EnPassant, None);
        assert!(is_move_legal(&pos, m), "White pawn can capture black pawn at b6");
    });

    named_test!(cannot_en_passant_the_next_play, {
        let mut pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);

        let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
        assert!(is_move_legal(&pos, m), "Black pawn can capture white pawn at e3");

        pos.make_move(m);
        let m = Move::new(Square::E3, Square::E4, MoveType::Normal, None);
        assert!(
            !is_move_legal(&pos, m),
            "White pawn can't capture black pawn because it didn't capture it immediately"
        );
    });

    named_test!(cannot_en_passant_if_pawn_moves_one_square, {
        let mut pos = Position::from(POSITION_1).unwrap();
        let m = Move::new(Square::E2, Square::E3, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);

        let m = Move::new(Square::A8, Square::A7, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);

        let m = Move::new(Square::E3, Square::E4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);

        let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(en_passant_cannot_leave_king_in_check, {
        let mut pos = Position::from("8/8/8/KP5r/1R3p1k/8/4P3/8 w - - 0 1").unwrap();
        let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);

        let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
        assert!(!is_move_legal(&pos, m));
    });

    named_test!(en_passant_can_capture_if_checker_is_the_pushed_pawn, {
        let mut pos = Position::from("8/8/8/KP1k3r/1R3p2/8/4P3/8 w - - 0 1").unwrap();
        let m = Move::new(Square::E2, Square::E4, MoveType::Normal, None);
        assert!(is_move_legal(&pos, m));
        pos.make_move(m);

        let m = Move::new(Square::F4, Square::E3, MoveType::EnPassant, None);
        assert!(is_move_legal(&pos, m));
    });
}
