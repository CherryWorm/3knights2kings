#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;

mod encoding;
mod moves;
mod search;
mod state;
mod tests;

use crate::search::*;
use crate::state::*;
use log::*;

fn main() {
    env_logger::init();

    let state = State {
        white_king: Position { x: 0, y: 5 },
        knights: [Position { x: 1, y: 0 }, Position { x: 2, y: 0 }, Position { x: 3, y: 0 }],
        black_king: Position { x: 0, y: 0 },
        target_field: Position { x: 0, y: 0 },
        white_to_move: false,
    };
    println!("{:?}\n{:?}", state.normalize(), State::unpack(state.pack()).normalize());

    minimax();
}
