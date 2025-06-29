#[test]
fn test_search_performance() {
    use bitboard_x::core::magic::warm_up_magic_tables;
    use bitboard_x::engine::Engine;
    use std::time::Instant;

    let now = Instant::now(); // Start timer

    warm_up_magic_tables();

    println!("Warm-up took: {:?}\n", now.elapsed());
    let now = Instant::now(); // Start timer for search

    let fen = "r1bqk2r/pppp1ppp/2n5/2b1p3/4P1n1/3P1N2/PPPNBPPP/R1BQ1RK1 b kq - 8 6";
    let mut engine = Engine::from_fen(fen).unwrap();
    let _mv = engine.best_move_depth(8).unwrap();

    println!("Search took: {:?}\n", now.elapsed());
}
