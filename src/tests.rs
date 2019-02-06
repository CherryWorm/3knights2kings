use crate::encoding::*;
use crate::state::*;
use std::thread;

#[test]
fn state_encoding_bijective() {
    let mut handles = Vec::new();
    for i in 0..8 {
        handles.push(thread::spawn(move || {
            for j in 0..8 {
                let white_king = i * 8 + j;
                for black_king in 0..64 {
                    println!("{}, {}", white_king, black_king);
                    if black_king == white_king {
                        continue;
                    }
                    for knight1 in 0..64 {
                        if knight1 == white_king || knight1 == black_king {
                            continue;
                        }
                        for knight2 in 0..64 {
                            if knight2 == white_king || knight2 == black_king || knight2 == knight1 {
                                continue;
                            }
                            for knight3 in 0..64 {
                                if knight3 == white_king || knight3 == black_king || knight3 == knight1 || knight3 == knight2 {
                                    continue;
                                }
                                for target_field in 0..28 {
                                    for white_to_move in 0..2 {
                                        let state = State {
                                            white_king: Position::from_u8(white_king),
                                            black_king: Position::from_u8(black_king),
                                            knights: [Position::from_u8(knight1), Position::from_u8(knight2), Position::from_u8(knight3)],
                                            target_field: Position::from_u8_rim(target_field),
                                            white_to_move: white_to_move == 1,
                                        };
                                        assert_eq!(state, State::unpack(state.pack()))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
