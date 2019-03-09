use rayon::ThreadPool;
use crate::state::Position;
use crate::state::State;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use chess::Board;
use crate::search::DRAW;
use crate::search::NOT_CALCULATED;
use chess::BoardStatus;
use chess::Color;
use chess::Square;
use chess::Rank;
use chess::File;
use chess::Piece;
use chess::MoveGen;
use indicatif::ProgressBar;

impl State {
    pub fn to_board(&self) -> Board {
        Board::from_fen(self.to_fen()).expect(self.to_lichess().as_str())
    }
    pub fn from_board(board: Board, target_field: Position) -> Self {
        let mut knights = [Position::from_u8(0), Position::from_u8(0), Position::from_u8(0)];
        let mut i = 0;
        for square in board.pieces(Piece::Knight) {
            knights[i] = Position::from_chess_square(square);
            i += 1;
        }
        assert_eq!(i, 3);
        State {
            white_to_move: board.side_to_move() == Color::White,
            white_king: Position::from_chess_square(board.king_square(Color::White)),
            black_king: Position::from_chess_square(board.king_square(Color::Black)),
            target_field,
            knights
        }
    }
}

impl Position {
    fn to_chess_square(self) -> Square {
        Square::make_square(Rank::from_index(self.y as usize), File::from_index(self.x as usize))
    }
    fn from_chess_square(square: Square) -> Self {
        Position { x: square.get_file().to_index() as u8, y: square.get_rank().to_index() as u8 }
    }
}


fn has_mate_in_one_on(board: &Board, target: &Square) -> bool {
    MoveGen::new_legal(&board).any(|m| board.make_move_new(m).status() == BoardStatus::Checkmate && board.king_square(Color::Black) == *target)
}

fn verify_state(dp: &Vec<u8>, state: State) -> bool {
    let board = state.to_board();
    let packed = state.pack();
    let dp_packed = dp[packed as usize];

    if dp_packed == NOT_CALCULATED || dp_packed == DRAW {
        if board.status() == BoardStatus::Checkmate && board.king_square(Color::Black) == state.target_field.to_chess_square() {
            println!("{} ({}) marked as draw, but black king is checkmated on target field {:?}!", state.to_lichess(), state.pack(), state.target_field);
            return false;
        }
        if board.status() == BoardStatus::Checkmate && board.king_square(Color::Black) != state.target_field.to_chess_square() {
            return true;
        }
        if state.white_to_move {
            for m in MoveGen::new_legal(&board) {
                let new_board = board.make_move_new(m);
                let new_state = State::from_board(new_board, state.target_field);
                let res = dp[new_state.pack() as usize];
                if res != NOT_CALCULATED && res != DRAW {
                    println!("{} marked as draw, but white is to play and child state {} after move {:?} is not marked as draw (target field: {:?})!", state.to_lichess(), new_state.to_lichess(), m, state.target_field);
                    return false;
                }
            }
        }
        else {
            let mut has_draw = false;
            for m in MoveGen::new_legal(&board) {
                let new_board = board.make_move_new(m);
                if new_board.pieces(Piece::Knight).popcnt() < 3 {
                    if has_mate_in_one_on(&new_board, &state.target_field.to_chess_square()) {
                        continue;
                    }
                    has_draw = true;
                    break
                }
                let new_state = State::from_board(new_board, state.target_field);
                let res = dp[new_state.pack() as usize];
                if res == NOT_CALCULATED || res == DRAW {
                    has_draw = true;
                    break
                }
            }
            if !has_draw && board.status() != BoardStatus::Stalemate {
                println!("{} marked as draw, but black is to play and has no move to achieve a draw (target field: {:?})!", state.to_lichess(), state.target_field);
                return false;
            }
        }
    }
    else if dp_packed == 0 {
        if state.white_to_move {
            println!("{} has mate in 0 on {:?} but white is to move?", state.to_lichess(), state.target_field);
            return false;
        }
        if board.status() != BoardStatus::Checkmate {
            println!("{} marked as checkmate in 0 on {:?}, but it's not!", state.to_lichess(), state.target_field);
            return false;
        }
        if board.king_square(Color::Black) != state.target_field.to_chess_square() {
            println!("{} marked as checkmate in 0, but black king is not on target field {:?} ({:?})!", state.to_lichess(), state.target_field.to_chess_square(), state.target_field);
            return false;
        }
    }
    else {
        if state.white_to_move {
            if dp_packed % 2 == 0 {
                println!("{} has mate in {} halfmoves on {:?}, but white is to move!", state.to_lichess(), dp_packed, state.target_field);
                return false;
            }
            if board.status() != BoardStatus::Ongoing {
                println!("{} is marked as mate in {} halfmoves on {:?}, but there are no moves that can be played!", state.to_lichess(), dp_packed, state.target_field);
                return false;
            }
            let mut min = 254;
            for m in MoveGen::new_legal(&board) {
                let new_board = board.make_move_new(m);
                let new_state = State::from_board(new_board, state.target_field);
                let res = dp[new_state.pack() as usize];
                if res < min {
                    min = res;
                }
            }
            if min != dp_packed - 1 {
                println!("{} ({}) is marked as mate in {} halfmoves on {:?}, but the minimum that can be achieved is {} + 1!", state.to_lichess(), packed, dp_packed, state.target_field, min);
                return false;
            }
        }
        else {
            if dp_packed % 2 == 1 {
                println!("{} has mate in {} halfmoves on {:?}, but black is to move!", state.to_lichess(), dp_packed, state.target_field);
                return false;
            }
            if board.status() != BoardStatus::Ongoing {
                println!("{} is marked as mate in {} halfmoves on {:?}, but there are no moves that can be played!", state.to_lichess(), dp_packed, state.target_field);
                return false;
            }
            let mut max = 0;
            for m in MoveGen::new_legal(&board) {
                let new_board = board.make_move_new(m);
                let res = if new_board.pieces(Piece::Knight).popcnt() < 3 {
                    if has_mate_in_one_on(&new_board, &state.target_field.to_chess_square()) {
                        1
                    }
                    else {
                        255
                    }
                }
                else {
                    let new_state = State::from_board(new_board, state.target_field);
                    dp[new_state.pack() as usize]
                };
                if res > max {
                    max = res;
                }
            }
            if max != dp_packed - 1 {
                println!("{} is marked as mate in {} halfmoves on {:?}, but the maximum that can be achieved is {} + 1 (a value of 254 or 255 indicates a draw)!", state.to_lichess(), dp_packed, state.target_field, max);
                return false;
            }
        }
    }
    return true;
}

pub fn verify(dp: &Vec<u8>, pool: ThreadPool) -> bool {
    println!("Verifying tablebase...");
    let result = AtomicBool::new(true);
    let bar = ProgressBar::new(16 * 63 * 28 * 37820);
    pool.install(|| {
        let result = &result;
        let bar = &bar;
        let counter = AtomicUsize::new(0);
        pool.scope(|s| {
            for target_field in 0..28 {
                let result = &result;
                let counter = &counter;
                let bar = &bar;
                s.spawn(move |_| {
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
                                        if !result.load(Ordering::SeqCst) {
                                            return;
                                        }
                                        if !verify_state(dp, state) {
                                            result.store(false, Ordering::SeqCst);
                                            return;
                                        }

                                        let state = State { white_to_move: true, ..state };
                                        if !state.covered_by_white().contains(black_king_pos) && !verify_state(dp, state)  {
                                            result.store(false, Ordering::SeqCst);
                                            return;
                                        }
                                        let val = counter.fetch_add(2, Ordering::SeqCst) + 2;
                                        if val % (16 * 63 * 28 * 2 * 37820 / 5000) < 2 {
                                            bar.set_position(val as u64);
                                            // println!("Verified {} / {} = {}% of states", val, 16 * 63 * 28 * 2 * 37820, val as f64 * 100.0 / (16.0 * 63.0 * 28.0 * 2.0 * 37820.0));
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });
        bar.finish();
        result.load(Ordering::SeqCst)
    })
}