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
        println!();
        game.display_alphabet(locale);
        let guess = game.ask_for_guess(locale);
        if game.game_is_over(&guess, locale) {
            break;
        }
    }
}
