use crate::moves;
use crate::state::{Position, State};
use std::collections::VecDeque;
use crate::search::Result::NotCalculated;
use crate::search::Result::MateIn;
use crate::search::Result::Draw;
use crate::tablebase::Tablebase;
use std::thread;
use crossbeam::channel;
use crossbeam::atomic::AtomicCell;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Result {
    NotCalculated,
    Draw,
    MateIn(u8),
}

enum Message {
    End,
    Calculate(State)
}

pub fn prev_layer_white(dp: &mut Vec<Result>, s: State, layer: u8, new_q: &mut VecDeque<u64>) -> () {
    for prev in s.previous_states() {
        let packed = prev.pack();
        if dp[packed as usize] == NotCalculated {
            dp[packed as usize] = MateIn(layer);
            new_q.push_back(packed);
        }
    }
}

pub fn prev_layer_black(dp: &mut Vec<Result>, outdeg: &mut Vec<i8>, s: State, layer: u8, new_q: &mut VecDeque<u64>) -> () {
    'outer: for prev in s.previous_states() {
        let prev_packed = prev.pack() as usize;

        if outdeg[prev_packed] == 0 {
            println!("\nALAAAAAAAAAAAAAAAAAARM\nState: {:?}, {:?}, {}\nPrevious: {:?}, {:?}, {}\n\n", s.to_lichess(), s, s.pack(), prev.to_lichess(), prev, s.pack());
            continue
        }

        if outdeg[prev_packed] < 0 {
            outdeg[prev_packed] = prev.next_states_count() as i8;
        }
        outdeg[prev_packed] -= 1;

        if outdeg[prev_packed] > 0 {
            continue;
        }

        for pos in ((prev.white_knights() & prev.black_king.king_moves()) / prev.covered_by_white()).iter() {
            if pos.to_u8() != 0 || prev.knights[1].to_u8() == 63 || prev.knights[2].to_u8() == 63 {
                dp[prev_packed] = Draw;
                continue 'outer
            }
            let dummy = State {
                black_king: Position::from_u8(0),
                knights: [prev.knights[1], prev.knights[2], Position::from_u8(63)],
                white_to_move: true,
                ..prev
            };
            if dp[dummy.pack() as usize] != MateIn(1) {
                dp[prev_packed] = Draw;
                continue 'outer
            }
        }

        dp[prev_packed as usize] = MateIn(layer);
        new_q.push_back(prev_packed as u64);
    }
}

pub fn retrograde_search() -> Tablebase {
    let mut dp = Vec::new();
    dp.resize(16 * 63 * 28 * 2 * 37820, Result::NotCalculated);
    let mut outdeg: Vec<i8> = Vec::new();
    outdeg.resize(16 * 63 * 28 * 2 * 37820, -1);

    let (send, receive) = channel::unbounded();
    let added = AtomicCell::new(0);

    for target_field in 0..28 {
        for white_king in 0..16 {
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
                                dp[packed as usize] = Result::MateIn(0);
                                send.send(Message(state));
                                added.fetch_add(1);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{} checkmate positions found", added.load());

    let mut white_to_play = false;
    let mut processed = added.load();
    let mut layer = 1;

    let (buffer_send, buffer_receive) = channel::unbounded();

    while added.load() != 0 {

        added.store(0);
        handles = vec![];

        for thread in 0..7 {
            handles.push(thread::spawn(move || {
                loop {
                    match receive.recv().unwrap() {
                        Message::End => break,
                        Message::Calculate(s) =>
                            if white_to_play {
                                prev_layer_black(&mut dp, &mut outdeg, s, layer, &mut new_q);
                            } else {
                                prev_layer_white(&mut dp, s, layer, &mut new_q);
                            }
                    }
                }
            }))
        }
        q = new_q;
        println!("{} positions found in layer {}. Processed {}/{} = {}% positions", q.len(), layer, processed, dp.len(), processed as f32 * 100.0 / dp.len() as f32);
        white_to_play = !white_to_play;
        layer += 1;
    }


    Tablebase{dp}
}
