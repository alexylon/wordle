use crate::wordle::WordleGame;

mod wordle;

fn main() {
    let mut game = WordleGame::new();
    // println!("{}", game.word);
    loop {
        game.display_guesses();
        let guess = game.ask_for_guess();
        if game.is_game_over(&guess) {
            break;
        }
    }
}
