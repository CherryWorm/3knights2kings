use crate::moves;
use crate::state::{Position, State};
use std::collections::VecDeque;
use crate::tablebase::Tablebase;
use std::thread;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicUsize;
use crossbeam::channel;
use crossbeam::atomic::AtomicCell;
use std::sync::atomic::Ordering;
use crossbeam::channel::Sender;
use rayon::ThreadPool;
use rayon::scope;

pub enum Message {
    End,
    Calculate(State)
}

pub const NOT_CALCULATED: u8 = 255;
pub const DRAW: u8 = 254;

pub fn prev_layer_white(dp: &Vec<AtomicU8>, s: State, layer: u8, sender: &Sender<Message>) -> () {
    for prev in s.previous_states() {
        let packed = prev.pack() as usize;
        let value = dp[packed].compare_and_swap(NOT_CALCULATED, layer, Ordering::SeqCst);
        if value == NOT_CALCULATED {
            sender.send(Message::Calculate(prev.normalize())).unwrap();
        }
    }
}

pub fn prev_layer_black(dp: &Vec<AtomicU8>, outdeg: &Vec<AtomicI8>, s: State, layer: u8, sender: &Sender<Message>) -> () {
    'outer: for prev in s.previous_states() {
        let prev_packed = prev.pack() as usize;

        if outdeg[prev_packed].load(Ordering::SeqCst) == 0 {
            println!("\nALAAAAAAAAAAAAAAAAAARM\nState: {:?}, {:?}, {}\nPrevious: {:?}, {:?}, {}\n\n", s.to_lichess(), s, s.pack(), prev.to_lichess(), prev, s.pack());
            continue
        }

        outdeg[prev_packed].compare_and_swap(-1, prev.next_states_count() as i8, Ordering::SeqCst);
        let outdeg = outdeg[prev_packed].fetch_sub(1, Ordering::SeqCst) - 1;

        if outdeg > 0 {
            continue;
        }

        for pos in ((prev.white_knights() & prev.black_king.king_moves()) / prev.covered_by_white()).iter() {
            if pos.to_u8() != 0 || prev.knights[1].to_u8() == 63 || prev.knights[2].to_u8() == 63 {
                dp[prev_packed].store(DRAW, Ordering::Relaxed);
                continue 'outer
            }
            let dummy = State {
                black_king: Position::from_u8(0),
                knights: [prev.knights[1], prev.knights[2], Position::from_u8(63)],
                white_to_move: true,
                ..prev
            };
            if dp[dummy.pack() as usize].load(Ordering::Relaxed) != 1 {
                dp[prev_packed].store(DRAW, Ordering::Relaxed);
                continue 'outer
            }
        }

        dp[prev_packed].store(layer, Ordering::Relaxed);
        sender.send(Message::Calculate(prev.normalize())).unwrap();
    }
}

fn fill_vec<T, F>(size: usize, f: F) -> Vec<T> where F: Fn() -> T {
    let mut res = vec![];
    for _ in 0..size {
        res.push(f());
    }
    res
}

fn generate_checkmates(dp: &Vec<AtomicU8>, sender: &Sender<Message>) -> usize {
    let added = AtomicUsize::new(0);
    scope(|s| {
        let added = &added;
        for target_field in 0..28 {
            s.spawn(move |_| {
                for white_king in 0..10 {
                    for black_king in 0..64 {
                        let white_king_pos = Position::from_u8_bottom_left(white_king);
                        let black_king_pos = Position::from_u8(black_king);
                        if white_king_pos.king_moves().contains(black_king_pos) || white_king_pos == black_king_pos {
                            continue;
                        }
                        for knight3 in 0..64 {
                            if knight3 == white_king_pos.to_u8() || knight3 == black_king {
                                continue;
                            }
                            for knight2 in 0..knight3 {
                                if knight2 == white_king_pos.to_u8() || knight2 == black_king {
                                    continue;
                                }
                                for knight1 in 0..knight2 {
                                    if knight1 == white_king_pos.to_u8() || knight1 == black_king {
                                        continue;
                                    }
                                    let state = State {
                                        white_king: white_king_pos,
                                        black_king: black_king_pos,
                                        knights: [Position::from_u8(knight1), Position::from_u8(knight2), Position::from_u8(knight3)],
                                        target_field: Position::from_u8_rim(target_field),
                                        white_to_move: false,
                                    };
                                    if state.is_mate() {
                                        let packed = state.pack();
                                        dp[packed as usize].store(0, Ordering::Relaxed);
                                        sender.send(Message::Calculate(state.normalize())).unwrap();
                                        added.fetch_add(1, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    let added = added.load(Ordering::SeqCst);
    println!("{} checkmate positions found", added);
    added
}

pub fn retrograde_search(pool: ThreadPool) -> Tablebase {
    pool.install( || {
        let dp = fill_vec(10 * 63 * 28 * 2 * 37820, || AtomicU8::new(NOT_CALCULATED));
        let outdeg = fill_vec(10 * 63 * 28 * 2 * 37820, || AtomicI8::new(-1));

        let (sender, receiver) = channel::unbounded();
        let (buffer_sender, buffer_receiver) = channel::unbounded();

        let mut white_to_play = false;
        let mut layer = 1;

        let mut added = generate_checkmates(&dp, &sender);
        let mut processed = added;

        while added != 0 {

            for _ in 0..16 {
                sender.send(Message::End).unwrap();
            }

            let receiver = receiver.clone();
            let buffer_sender_clone = buffer_sender.clone();
            let dp = &dp;
            let outdeg = &outdeg;

            scope(move |s| {
                for _ in 0..16 {
                    let receiver = receiver.clone();
                    let buffer_sender = buffer_sender_clone.clone();
                    s.spawn(move |_| {
                        loop {
                            match receiver.recv().unwrap() {
                                Message::End => break,
                                Message::Calculate(s) =>
                                    if white_to_play {
                                        prev_layer_black(dp, outdeg, s, layer, &buffer_sender);
                                    } else {
                                        prev_layer_white(dp, s, layer, &buffer_sender);
                                    }
                            }
                        }
                    })
                }
            });

            buffer_sender.send(Message::End).unwrap();

            processed += added;
            added = 0;
            loop {
                match buffer_receiver.recv().unwrap() {
                    Message::End => break,
                    calc => {
                        added += 1;
                        sender.send(calc).unwrap();
                    }
                }
            }

            println!("{} positions found in layer {}. Processed {}/{} = {}% positions", added, layer, processed, dp.len(), processed as f32 * 100.0 / dp.len() as f32);
            white_to_play = !white_to_play;
            layer += 1;
        }


        Tablebase{dp: dp.iter().map(|i| i.load(Ordering::Relaxed)).collect()}
    })
}
