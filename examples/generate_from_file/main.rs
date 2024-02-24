//! Example reading text from a file, generating a chain, and then printing `n` amount of tokens.
//!
//! Usage: generate_from_file <FILE_PATH> <N>
//! Using `cargo run`: `cargo run --example generate_from_file -- <FILE_PATH> <N>`

use markovish::Chain;

use rand::thread_rng;
use std::{path::PathBuf, process::exit};

const USAGE: &str = "Usage: generate_from_file <FILE_PATH> <N>";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("{USAGE}");
        exit(1);
    }

    let text =
        std::fs::read_to_string(PathBuf::from(args[1].clone())).expect("could not read file");
    let mut cb = Chain::builder();
    cb.feed_str(&text).expect("could not feed string");
    let chain = cb.build();
    let start = chain.start_tokens(&mut thread_rng()).unwrap();
    let gen_text = chain
        .generate_n_tokens(
            &mut thread_rng(),
            &start.as_ref(),
            args[2]
                .parse()
                .expect("did not provide a valid token number"),
        )
        .expect("failed to generate text");
    println!("{}", gen_text.join(""));
}
