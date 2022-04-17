use colored::*;
use bracket_random::prelude::RandomNumberGenerator;
use std::collections::HashSet;

const ALL_WORDS: &str = include_str!("words_en.txt");
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
                    format!("{}", c).green()
                } else if self.word.chars().any(|wc| wc == c) {
                    format!("{}", c).bright_yellow()
                } else {
                    self.guessed_letters.insert(c);
                    format!("{}", c).red()
                };
                print!("{}", display);
            });
            println!();
        })
    }

    fn display_invalid_letters(&self) {
        if !self.guessed_letters.is_empty() {
            print!("Letters not in the word: ");
            self.guessed_letters.iter()
                .for_each(|letter| print!("{} ", letter));
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
            std::io::stdin().read_line(&mut guess).unwrap();
            guess = sanitize_word(&guess);
            println!("{}", guess);
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

    pub(crate) fn is_game_over(&self, guess: &str) -> bool {
        let n_tries = self.guesses.len();
        if guess == self.word {
            println!("{}", format!("Correct! You guessed the word in {} tries.", n_tries).bright_green());
            true
        } else if n_tries >= MAX_TRIES {
            println!("{}", format!("You ran out of tries! The word was {}", self.word).bright_red());
            true
        } else {
            false
        }
    }
}
