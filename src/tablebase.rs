use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::io::Read;
use crate::search::{retrograde_search, DRAW, NOT_CALCULATED};
use rayon::ThreadPoolBuilder;
use crate::verification::verify;
use indicatif::ProgressBar;
use crate::state::{State, Position};
use chess::{ChessMove, MoveGen, Board};

pub struct Tablebase {
    pub dp: Vec<u8>
}

pub enum Value {
    MateIn(u8),
    Draw
}

pub struct Evaluation {
    pub best_moves: Vec<ChessMove>,
    pub value: Value
}

impl Tablebase {
    pub fn write_to_disk(&self, file: File) {
        println!("Writing tablebase to disk...");
        let mut writer = BufWriter::new(&file);
        let mut i = 0;
        let bar = ProgressBar::new(self.dp.len()as u64);
        for r in &self.dp {
            writer.write(&[*r]).unwrap();
            i += 1;
            if i == 2000000 {
                bar.inc(2000000);
                i = 0;
            }
        }
        bar.finish();
    }

    pub fn read_from_disk(file: File) -> Result<Self, &'static str> {
        println!("Reading tablebase from disk...");
        if file.metadata().unwrap().len() != 16 * 63 * 28 * 2 * 37820 {
            Err("Tablebase has the wrong size!")
        }
        else {
            let reader = BufReader::new(&file);
            let mut dp = Vec::new();
            let bar = ProgressBar::new(file.metadata().unwrap().len());
            let mut i = 0;
            for r in reader.bytes() {
                dp.push(r.unwrap());
                i += 1;
                if i == 2000000 {
                    bar.inc(2000000);
                    i = 0;
                }
            }
            bar.finish();
            Ok(Tablebase { dp })
        }
    }

    pub fn normalize(&mut self) {
        for i in 0..self.dp.len() {
            if self.dp[i] == NOT_CALCULATED {
                self.dp[i] = DRAW;
            }
        }
    }

    pub fn print_stats(&self) {
        let mut mate = 0;
        let mut draw = 0;
        let mut not_calculated = 0;
        for s in &self.dp {
            match *s {
                DRAW => draw += 1,
                NOT_CALCULATED => not_calculated += 1,
                _ => mate += 1
            }
        }
        let percent = 100.0 / self.dp.len() as f32;
        println!("{} ({}%) mate, {} ({}%) draw, {} ({}%) not calculated from {} total", mate, mate as f32 * percent, draw, draw as f32  * percent, not_calculated, not_calculated as f32  * percent, self.dp.len())
    }

    pub fn generate(threads: usize) -> Self {
        retrograde_search(ThreadPoolBuilder::new().num_threads(threads).build().unwrap())
    }

    pub fn verify(&self, threads: usize) -> bool {
        verify(&self.dp, ThreadPoolBuilder::new().num_threads(threads).build().unwrap())
    }

    pub fn eval(&self, board: Board, target: Position) -> Evaluation {
        let s = State::from_board(board, target);
        if !s.target_field.is_on_rim() {
            Evaluation {best_moves: vec![], value: Value::Draw}
        }
        else {
            let dp_s = self.dp[s.pack() as usize];
            println!("{}", dp_s);
            let best_moves = if s.white_to_move {
                MoveGen::new_legal(&board)
                    .filter(|m| { let next = self.dp[State::from_board(board.make_move_new(*m), s.target_field).pack() as usize]; println!("{:?}: {}", m, next); dp_s == DRAW || next == dp_s - 1 })
                    .collect()
            }
            else {
                println!("{}", dp_s);
                MoveGen::new_legal(&board)
                    .filter(|m| { let next = self.dp[State::from_board(board.make_move_new(*m), s.target_field).pack() as usize]; println!("{:?}: {}", m, next); (dp_s == DRAW && next == DRAW) || (dp_s != DRAW && dp_s - 1 == next) })
                    .collect()
            };
            let value = if dp_s == DRAW { Value::Draw } else { Value::MateIn(dp_s) };
            Evaluation {best_moves, value}
        }
    }
}