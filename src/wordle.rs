use colored::*;
use bracket_random::prelude::RandomNumberGenerator;
use std::collections::HashSet;
use fluent::{FluentBundle, FluentValue, FluentResource, FluentArgs, FluentMessage};
use std::fs::File;
use std::{fs, io};
use std::io::prelude::*;
use std::path::Path;
use std::env;
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
// Used to provide a locale for the bundle.
use unic_langid::{langid, LanguageIdentifier};

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

    fn display_invalid_letters(&self, locale: &str) {
        let mut letters: Vec<char> = self.guessed_letters.clone().into_iter().collect();
        if !letters.is_empty() {
            print!("{} ", get_message(&get_bundle(&locale), "invalid-letters-message"));
            letters.sort();
            letters.iter()
                .for_each(|letter| print!("{} ", format!("{}", letter).bold()));
            println!();
        }
    }

    pub fn ask_for_guess(&mut self, locale: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("word_length", FluentValue::from(WORD_LENGTH));
        println!("{}", format!("{}", get_message_args(&get_bundle(&locale), "enter-word-message", &args)).cyan());
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

    pub(crate) fn is_game_over(&mut self, guess: &str, locale: &str) -> bool {
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

/// We need a generic file read helper function to read the localization resource file.
fn read_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// This helper function allows us to read the list of available locales
///
/// It is expected that every directory inside it has a name that is a valid BCP47 language tag.
fn get_available_locales() -> Result<Vec<LanguageIdentifier>, io::Error> {
    let mut locales = vec![];

    let mut dir = env::current_dir()?;
    dir.push("src/translations");
    let res_dir = fs::read_dir(dir)?;
    for entry in res_dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name) = name.to_str() {
                        if let Ok(langid) = name.parse() {
                            locales.push(langid);
                        } else {
                            eprintln!("Parsing failed.");
                        }
                    }
                }
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
        let mut full_path = env::current_dir().expect("Failed to retrieve current dir.");
        full_path.push("src/translations");
        full_path.push(current_locale.to_string());
        full_path.push(path);
        let source = read_file(&full_path).expect("Failed to read file.");
        let resource = FluentResource::try_new(source).expect("Could not parse an FTL string.");
        bundle
            .add_resource(resource)
            .expect("Failed to add FTL resources to the bundle.");
    }

    bundle
}
