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

fn main() {
    let state = State::from_fen("k7/4K3/1NN5/8/8/4N3/8/8 b - -", Position {x: 1, y: 0});
    println!("{}, {:?}", state.to_lichess(), state.pack());

    env_logger::init();

    let tablebase = retrograde_search();

    tablebase.write_to_disk(File::create("tb.raw").expect("Couldn't open tablebase file"));
    tablebase.print_stats();
}
