#![feature(integer_atomics)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate crossbeam;

mod encoding;
mod moves;
mod search;
mod state;
mod tests;
mod tablebase;

use crate::search::*;
use crate::state::*;
use log::*;
use std::fs::File;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use std::time::Instant;

fn main() {
    env_logger::init();

    let now = Instant::now();
    let tablebase = retrograde_search(ThreadPoolBuilder::new().num_threads(8).build().unwrap());
    println!("Generated tablebase in {} seconds", now.elapsed().as_secs());

    tablebase.write_to_disk(File::create("tb.raw").expect("Couldn't open tablebase file"));
    tablebase.print_stats();
}
