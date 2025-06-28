use bitboard_x::engine::Engine;

#[test]
fn test_search_performance() {
    let fen = "r1bqk2r/pppp1ppp/2n5/2b1p3/4P1n1/3P1N2/PPPNBPPP/R1BQ1RK1 b kq - 8 6";
    let mut engine = Engine::from_fen(fen).unwrap();
    let _mv = engine.best_move_depth(9).unwrap();
}
