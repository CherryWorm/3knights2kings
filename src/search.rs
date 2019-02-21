use crate::moves;
use crate::state::{Position, State};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Result {
    NotCalculated,
    Calculating,
    Draw,
    MateIn(u8),
}

fn btm(s: State, dp: &Vec<Result>) -> Result {
    if dp[s.pack() as usize] != Result::NotCalculated {
        return dp[s.pack() as usize];
    }
    unimplemented!()
}

fn wtm(s: State, dp: &Vec<Result>) -> Result {
    unimplemented!()
}

pub fn minimax() -> Vec<Result> {
    let mut dp = Vec::new();
    dp.resize(16 * 63 * 28 * 2 * 37820, Result::NotCalculated);

    let mut q = VecDeque::new();

    for target_field in 0..28 {
        for white_king in 0..16 {
            for black_king in 0..64 {
                let white_king_pos = Position::from_u8_bottom_left(white_king);
                let black_king_pos = Position::from_u8(black_king);
                if white_king_pos.king_moves().contains(black_king_pos) || white_king == black_king {
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
                                q.push_back(packed);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{} checkmate positions found", q.len());

    dp
}
