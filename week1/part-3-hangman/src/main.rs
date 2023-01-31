// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn hello() {
    println!("Welcome to Hangman!");
    println!("You have {} incorrect guesses.", NUM_INCORRECT_GUESSES);
}

fn show_word(word: &str, guessed_letters: &Vec<char>) {
    print!("The Word so far is ");
    for c in word.chars() {
        if guessed_letters.contains(&c) {
            print!("{}", c);
        } else {
            print!("-");
        }
    }
    println!("");
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);
    
    hello();
    let mut guessed_letters: Vec<char> = Vec::new();
    loop {
        show_word(&secret_word, &guessed_letters);

        // get input
        print!("Guess a letter: ");
        io::stdout().flush().unwrap();
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Failed to read line");
        let guess_char = guess.chars().next().unwrap();
        guessed_letters.push(guess_char);

        // check
        if secret_word_chars.iter().all(|c| guessed_letters.contains(c)) {
            println!("You win!");
            break;
        }
        if guessed_letters.len() == NUM_INCORRECT_GUESSES as usize {
            println!("You lose!");
            break;
        }
    }
}
