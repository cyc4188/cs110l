use std::env;
use std::process;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    let mut f = File::open(filename).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    println!("{} has {} words", filename, contents.split_whitespace().count());
}

