#![feature(integer_atomics)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod encoding;
mod moves;
mod search;
mod state;
mod tests;
mod tablebase;
mod verification;

use crate::search::*;
use crate::state::*;
use log::*;
use std::fs::File;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use std::time::Instant;
use crate::tablebase::Tablebase;
use std::env;
use indicatif::HumanDuration;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 1 && args[1] == "validate" {
        let tablebase = Tablebase::read_from_disk(File::open("tb.raw").unwrap());
        let now = Instant::now();
        let verify = tablebase.verify(7);
        println!("Verified tablebase in {}. Result: {}", HumanDuration(now.elapsed()), verify);
    }
    else {
        let now = Instant::now();
        let tablebase = Tablebase::generate(7);
        println!("Generated tablebase in {}", HumanDuration(now.elapsed()));
        tablebase.write_to_disk(File::create("tb.raw").expect("Couldn't open tablebase file"));
        tablebase.print_stats();
    }
}
