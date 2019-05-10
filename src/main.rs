#![feature(integer_atomics)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate text_io;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

mod encoding;
mod moves;
mod search;
mod state;
mod tablebase;
mod verification;
mod webserver;

use crate::state::*;
use std::fs::File;
use std::time::Instant;
use crate::tablebase::Tablebase;
use clap::{Arg, App, SubCommand};
use crate::tablebase::Value::MateIn;
use std::path::Path;
use crate::webserver::start_server;


fn gen(threads: usize, file: File) {
    let start = Instant::now();
    let tb = Tablebase::generate(threads);
    println!("Tablebase generated in {} seconds", start.elapsed().as_secs());
    tb.write_to_disk(file);
}

fn validate(threads: usize, file: File) {
    let tb = Tablebase::read_from_disk(file).unwrap();
    let start = Instant::now();
    if tb.verify(threads) {
        println!("The tablebase is consistent!");
    }
    else {
        println!("The tablebase is not consistent!");
    }
    println!("Tablebase verified in {} seconds", start.elapsed().as_secs());
}

fn eval(file: File) {
    let mut tb = Tablebase::read_from_disk(file).unwrap();
    tb.normalize();
    loop {
        let fen: String = read!("{}\n");
        let target = read!("{}\n");
        let s = State::from_fen(&fen, Position::from_string(&target).unwrap()).unwrap();
        let eval = tb.eval(&s);
        if let MateIn(n) = eval.value {
            println!("White has mate in {} half moves. Best moves:", n);
            let mut first = true;
            for m in &eval.best_moves {
                if first {
                    first = false
                }
                else {
                    print!(", ")
                }
                print!("{}{}", m.get_source().to_string(), m.get_dest().to_string())
            }
        }
        else {
            println!("The position is an objective draw. Best moves:")
        }
    }
}

fn server(file: File) {
    let mut tb = Tablebase::read_from_disk(file).unwrap();
    tb.normalize();
    start_server(tb);
}



fn main() {

    let matches = App::new("3 Knights 2 Kings")
        .subcommand(SubCommand::with_name("gen")
            .about("generates the tablebase")
            .arg(Arg::with_name("output")
                .help("The output file")
                .required(true)
                .index(1))
            .arg(Arg::with_name("threads")
                .short("t")
                .long("threads")
                .takes_value(true)
                .value_name("n")
                .required(false)
                .default_value("7")
                .help("The amount of threads to use")))
        .subcommand(SubCommand::with_name("validate")
            .about("validates the tablebase (this takes a long time)")
            .arg(Arg::with_name("input")
                .help("The tablebase file")
                .required(true)
                .index(1))
            .arg(Arg::with_name("threads")
                .short("t")
                .long("threads")
                .takes_value(true)
                .value_name("n")
                .required(false)
                .default_value("7")
                .help("The amount of threads to use")))
        .subcommand(SubCommand::with_name("eval")
            .about("reads FENs and target squares (each on their own line) from stdin and evaluates the positions")
            .arg(Arg::with_name("input")
                .help("The tablebase file")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("server")
            .about("Launches a webserver with a simple ui")
            .arg(Arg::with_name("input")
                .help("The tablebase file")
                .required(true)
                .index(1)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("gen") {
        let file = File::create(Path::new(matches.value_of("output").unwrap())).unwrap();
        let threads = matches.value_of("threads").unwrap().parse().unwrap();
        gen(threads, file);
    }
    else if let Some(matches) = matches.subcommand_matches("validate") {
        let file = File::open(Path::new(matches.value_of("input").unwrap())).unwrap();
        let threads = matches.value_of("threads").unwrap().parse().unwrap();
        validate(threads, file);
    }
    else if let Some(matches) = matches.subcommand_matches("eval") {
        let file = File::open(Path::new(matches.value_of("input").unwrap())).unwrap();
        eval(file);
    }
    else if let Some(matches) = matches.subcommand_matches("server") {
        let file = File::open(Path::new(matches.value_of("input").unwrap())).unwrap();
        server(file);
    }


}
