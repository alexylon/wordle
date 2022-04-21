use colored::*;
use bracket_random::prelude::RandomNumberGenerator;
use std::collections::HashSet;
use fluent::{FluentBundle, FluentValue, FluentResource, FluentArgs};
use std::{io};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
// Used to provide a locale for the bundle.
use unic_langid::{langid, LanguageIdentifier};
use include_dir::{include_dir, Dir};

// include file to binaries
const ALL_WORDS: &str = include_str!("resources/words_bg_short.txt");
// Include dir to binaries
static TRANSLATIONS_DIR: Dir = include_dir!("./src/translations");
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
    invalid_letters: HashSet<char>,
    valid_letters: HashSet<char>,
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
            invalid_letters: HashSet::new(),
            valid_letters: HashSet::new(),
            guesses: Vec::new(),
        }
    }

    pub fn display_guesses(&mut self) {
        self.guesses.iter().enumerate().for_each(|(guess_number, guess)| {
            print!("{}: ", guess_number + 1);
            guess.chars().enumerate().for_each(|(pos, c)| {
                match self.word.chars().nth(pos) {
                    None => { eprintln!("No char found!"); }
                    Some(character) => {
                        let display = if character == c {
                            self.valid_letters.insert(c);
                            format!("{}", c).truecolor(0, 170, 120).bold()
                        } else if self.word.chars().any(|wc| wc == c) {
                            self.valid_letters.insert(c);
                            format!("{}", c).bright_yellow().bold()
                        } else {
                            self.invalid_letters.insert(c);
                            format!("{}", c).bold().truecolor(210, 0, 0)
                        };
                        print!("{}", display);
                    }
                }
            });
            println!();
        })
    }

    fn display_invalid_letters(&self, locale: &str) {
        let mut invalid_letters: Vec<char> = self.invalid_letters.clone().into_iter().collect();
        if !invalid_letters.is_empty() {
            print!("{} ", get_message(&get_bundle(&locale), "invalid-letters-message"));
            invalid_letters.sort();
            invalid_letters.iter()
                .for_each(|letter| print!("{} ", format!("{}", letter).bold()));
            println!();
        }
    }

    pub fn display_alphabet(&mut self, locale: &str) {
        let alphabet = get_message(&get_bundle(&locale), "alphabet");
        let valid_letters: Vec<char> = self.valid_letters.clone().into_iter().collect();
        let invalid_letters: Vec<char> = self.invalid_letters.clone().into_iter().collect();

        for c_alphabet in alphabet.chars() {
            let display = if valid_letters.contains(&c_alphabet) {
                format!("{} ", c_alphabet).green().bold()
            } else if invalid_letters.contains(&c_alphabet) {
                format!("{} ", c_alphabet).truecolor(210, 0, 0).bold()
            } else {
                format!("{} ", c_alphabet).bold()
            };

            print!("{}", display);
        }

        println!();
    }

    pub fn ask_for_guess(&mut self, locale: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("word_length", FluentValue::from(WORD_LENGTH));
        println!("\n{}", format!("{}", get_message_args(&get_bundle(&locale), "enter-word-message", &args)).cyan());
        self.display_invalid_letters(locale);
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
                println!("{}", format!("{}", get_message_args(&get_bundle(locale), "length-warning", &args)).red());
            } else if !self.dictionary.iter().any(|word| word == &guess) {
                args.set("guess", FluentValue::from(guess.clone()));
                println!("{}", format!("{}", get_message_args(&get_bundle(locale), "not-found-warning", &args)).red());
            } else {
                self.guesses.push(guess.clone());
                valid_guess = true;
            }
        }
        guess
    }

    pub(crate) fn game_is_over(&mut self, guess: &str, locale: &str) -> bool {
        let mut args = FluentArgs::new();
        let n_tries = self.guesses.len();
        args.set("word", FluentValue::from(self.word.clone()));
        if guess == self.word {
            args.set("n_tries", FluentValue::from(n_tries));
            println!("{}", format!("{}", get_message_args(&get_bundle(locale), "correct-message", &args)).truecolor(0, 170, 120).bold());
            true
        } else if n_tries >= MAX_TRIES {
            self.display_guesses();
            println!("{}", format!("{}", get_message_args(&get_bundle(locale), "failed-message", &args)).bright_red().bold());
            true
        } else {
            false
        }
    }
}

// I18N

/// This helper function allows us to read the list of available locales
///
/// It is expected that every directory inside it has a name that is a valid BCP47 language tag.
fn get_available_locales() -> Result<Vec<LanguageIdentifier>, io::Error> {
    let mut locales = vec![];
    let dirs = TRANSLATIONS_DIR.dirs();
    for entry in dirs {
        if let Some(name) = entry.path().to_str() {
            if let Ok(langid) = name.parse() {
                locales.push(langid);
            } else {
                eprintln!("Parsing failed.");
            }
        }
    }

    return Ok(locales);
}

static L10N_RESOURCES: &[&str] = &["translation.ftl"];

fn get_message(bundle: &FluentBundle<FluentResource>, message_id: &str) -> String {
    get_value(bundle, None, message_id)
}

fn get_message_args(bundle: &FluentBundle<FluentResource>, message_id: &str, args: &FluentArgs) -> String {
    get_value(bundle, Some(args), message_id)
}

fn get_value(bundle: &FluentBundle<FluentResource>, args: Option<&FluentArgs>, message_id: &str) -> String {
    match bundle.get_message(message_id) {
        None => { message_id.to_string() }
        Some(message) => {
            match message.value() {
                Some(pattern) => {
                    let value = bundle.format_pattern(&pattern, args, &mut vec![]);
                    format!("{}", value)
                }
                None => { message_id.to_string() }
            }
        }
    }
}

fn get_bundle(locale: &str) -> FluentBundle<FluentResource> {
    let mut requested: Vec<LanguageIdentifier> = vec![];
    if let Ok(langid) = locale.parse() {
        requested.push(langid);
    } else {
        println!("Parsing failed: ParserError(InvalidLanguage)")
    }

    // Negotiate it against the available ones
    let default_locale = langid!("en");
    let available = get_available_locales().expect("Retrieving available locales failed.");
    let resolved_locales = negotiate_languages(
        &requested,
        &available,
        Some(&default_locale),
        NegotiationStrategy::Filtering,
    );

    let current_locale = resolved_locales
        .get(0)
        .cloned()
        .expect("At least one locale should match.");

    // Create a new Fluent FluentBundle using the resolved locales.
    let mut bundle = FluentBundle::new(resolved_locales.into_iter().cloned().collect());

    // Load the localization resource
    for path in L10N_RESOURCES {
        match TRANSLATIONS_DIR.get_file(format!("{}/{}", current_locale, path)) {
            None => { eprintln!("No file found!") }
            Some(file) => {
                let source = match std::str::from_utf8(file.contents()) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                let resource = FluentResource::try_new(source.to_string()).expect("Could not parse an FTL string.");
                bundle
                    .add_resource(resource)
                    .expect("Failed to add FTL resources to the bundle.");
            }
        }
    }

    bundle
}
