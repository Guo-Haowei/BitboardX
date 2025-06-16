use bitboard_x::engine::utils;
use bitboard_x::game::Game;

fn main() {
    let mut game = Game::new();

    loop {
        println!("{}", utils::debug_string(game.position()));

        if game.game_over() {
            println!("Game over!");
            break;
        }

        game.tick();
    }
}
