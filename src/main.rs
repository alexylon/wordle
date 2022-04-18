use crate::wordle::WordleGame;

mod wordle;

fn main() {
    run_game("bg");
}

fn run_game(locale: &str) {
    let mut game = WordleGame::new();
    // println!("{}", game.word);
    loop {
        game.display_guesses();
        let guess = game.ask_for_guess(locale);
        if game.is_game_over(&guess, locale) {
            break;
        }
    }
}
