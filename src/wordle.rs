use colored::*;
use bracket_random::prelude::RandomNumberGenerator;
use std::collections::HashSet;

const ALL_WORDS: &str = include_str!("resources/words_bg_short.txt");
const WORD_LENGTH: usize = 5;
const MAX_TRIES: usize = 6;

fn sanitize_word(word: &str) -> String {
    word.trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect()
}

fn words_list() -> Vec<String> {
    ALL_WORDS
        .split('\n')
        .skip(2)
        .map(sanitize_word)
        .filter(|line| line.chars().count() == WORD_LENGTH)
        .collect()
}

#[derive(Clone)]
pub struct WordleGame {
    dictionary: Vec<String>,
    pub(crate) word: String,
    guessed_letters: HashSet<char>,
    guesses: Vec<String>,
}

impl WordleGame {
    pub(crate) fn new() -> Self {
        let mut rng = RandomNumberGenerator::new();
        let dictionary = words_list();
        let word = rng.random_slice_entry(&dictionary)
            .expect("No word found")
            .clone();

        Self {
            dictionary,
            word: word.to_string(),
            guessed_letters: HashSet::new(),
            guesses: Vec::new(),
        }
    }

    pub fn display_guesses(&mut self) {
        self.guesses.iter().enumerate().for_each(|(guess_number, guess)| {
            print!("{}: ", guess_number + 1);
            guess.chars().enumerate().for_each(|(pos, c)| {
                let display = if self.word.chars().nth(pos).unwrap() == c {
                    format!("{}", c).truecolor(0, 170, 120).bold()
                } else if self.word.chars().any(|wc| wc == c) {
                    format!("{}", c).bright_yellow().bold()
                } else {
                    self.guessed_letters.insert(c);
                    format!("{}", c).bold().truecolor(210, 0, 0)
                };
                print!("{}", display);
            });
            println!();
        })
    }

    fn display_invalid_letters(&self) {
        let mut letters: Vec<char> = self.guessed_letters.clone().into_iter().collect();
        if !letters.is_empty() {
            print!("Letters not in the word: ");
            letters.sort();
            letters.iter()
                .for_each(|letter| print!("{} ", format!("{}", letter).bold()));
            println!();
        }
    }

    pub fn ask_for_guess(&mut self) -> String {
        println!(
            "{}",
            format!("Enter your word guess ({} letters) and press ENTER", WORD_LENGTH).cyan()
        );
        self.display_invalid_letters();
        let mut guess = String::new();
        let mut valid_guess = false;
        while !valid_guess {
            guess = String::new();
            match std::io::stdin().read_line(&mut guess) {
                Ok(_) => {}
                Err(e) => { eprintln!("Error: {}", e); }
            }
            guess = sanitize_word(&guess);
            // println!("{}", guess);
            if guess.chars().count() != WORD_LENGTH {
                println!("{}", format!("Your guess must be {} letters.", WORD_LENGTH).red())
            } else if !self.dictionary.iter().any(|word| word == &guess) {
                println!("{}", format!("{} isn't in the dictionary.", guess).red());
            } else {
                self.guesses.push(guess.clone());
                valid_guess = true;
            }
        }
        guess
    }

    pub(crate) fn is_game_over(&mut self, guess: &str) -> bool {
        let n_tries = self.guesses.len();
        if guess == self.word {
            println!("{}", format!("Correct! You guessed the word {} in {} tries.", self.word.truecolor(0, 170, 120).bold(), n_tries).bright_green());
            true
        } else if n_tries >= MAX_TRIES {
            self.display_guesses();
            println!("{}", format!("You ran out of tries! The word was {}", self.word.bold()).bright_red());
            true
        } else {
            false
        }
    }
}
