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

pub fn minimax() -> Vec<Result> {
    let mut dp = Vec::new();
    dp.resize(16 * 63 * 28 * 2 * 37820, Result::NotCalculated);

    let mut q = VecDeque::new();
    let mut i = 0;

    for target_field in 0..28 {
        for white_king in 0..16 {
            for black_king in 0..64 {
                if white_king == black_king {
                    continue;
                }
                for knight3 in 0..64 {
                    if knight3 == white_king || knight3 == black_king {
                        continue;
                    }
                    for knight2 in 0..knight3 {
                        if knight2 == white_king || knight2 == black_king {
                            continue;
                        }
                        for knight1 in 0..knight2 {
                            if knight1 == white_king || knight1 == black_king {
                                continue;
                            }
                            let state = State {
                                white_king: Position::from_u8_bottom_left(white_king),
                                black_king: Position::from_u8(black_king),
                                knights: [Position::from_u8(knight1), Position::from_u8(knight2), Position::from_u8(knight3)],
                                target_field: Position::from_u8_rim(target_field),
                                white_to_move: false,
                            };
                            i += 1;
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

    info!("{}/{} = {}%", q.len(), i, q.len() * 100 / i);

    dp
}
