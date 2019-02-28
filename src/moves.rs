use crate::state::{Position, State};
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PossibleMoves(u64);

impl PossibleMoves {
    fn dif(&self, other: PossibleMoves) -> Self {
        let a = self.0;
        let b = other.0;
        PossibleMoves(a & (a ^ b))
    }

    fn union(&self, other: PossibleMoves) -> Self {
        let a = self.0;
        let b = other.0;
        PossibleMoves(a | b)
    }

    fn intersection(&self, other: PossibleMoves) -> Self {
        let a = self.0;
        let b = other.0;
        PossibleMoves(a & b)
    }

    pub fn contains(&self, pos: Position) -> bool {
        (self.0 >> pos.to_u8()) & 1 == 1
    }

    pub fn iter(&self) -> PossibleMovesIterator {
        PossibleMovesIterator(self.0, 0)
    }

    pub fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }

    fn from_position(pos: Position) -> Self {
        PossibleMoves(1 << pos.to_u8() as u64)
    }
    fn empty() -> Self {
        PossibleMoves(0)
    }
}

impl ops::BitOr<PossibleMoves> for PossibleMoves {
    type Output = PossibleMoves;
    fn bitor(self, other: PossibleMoves) -> PossibleMoves {
        self.union(other)
    }
}

impl ops::BitAnd<PossibleMoves> for PossibleMoves {
    type Output = PossibleMoves;
    fn bitand(self, other: PossibleMoves) -> PossibleMoves {
        self.intersection(other)
    }
}


impl ops::Div<PossibleMoves> for PossibleMoves {
    type Output = PossibleMoves;
    fn div(self, other: PossibleMoves) -> PossibleMoves {
        self.dif(other)
    }
}

pub struct PossibleMovesIterator(u64, u8);

impl Iterator for PossibleMovesIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        }
        else {
            while self.0 & 1 == 0 {
                self.0 >>= 1;
                self.1 += 1;
            }
            let res = Position::from_u8(self.1);
            self.0 >>= 1;
            self.1 += 1;
            Some(res)
        }
    }
}



fn add_if_not_out_of_bounds(m: PossibleMoves, pos: Position, dx: i16, dy: i16) -> PossibleMoves {
    if !pos.is_out_of_bounds(dx, dy) {
        m | PossibleMoves::from_position(pos.add(dx, dy))
    } else {
        m
    }
}

lazy_static! {
    static ref knight_moves: Vec<PossibleMoves> = {
        let mut moves = Vec::new();
        for i in 0..64 {
            let pos = Position::from_u8(i);
            let m = PossibleMoves::empty();
            let m = add_if_not_out_of_bounds(m, pos, -1, 2);
            let m = add_if_not_out_of_bounds(m, pos, 1, 2);
            let m = add_if_not_out_of_bounds(m, pos, -1, -2);
            let m = add_if_not_out_of_bounds(m, pos, 1, -2);
            let m = add_if_not_out_of_bounds(m, pos, -2, 1);
            let m = add_if_not_out_of_bounds(m, pos, 2, 1);
            let m = add_if_not_out_of_bounds(m, pos, -2, -1);
            let m = add_if_not_out_of_bounds(m, pos, 2, -1);
            moves.push(m);
        }
        moves
    };
    static ref king_moves: Vec<PossibleMoves> = {
        let mut moves = Vec::new();
        for i in 0..64 {
            let pos = Position::from_u8(i);
            let mut m = PossibleMoves::empty();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx != 0 || dy != 0 {
                        m = add_if_not_out_of_bounds(m, pos, dx, dy);
                    }
                }
            }
            moves.push(m);
        }
        moves
    };
}

impl Position {
    pub fn king_moves(&self) -> PossibleMoves {
        king_moves[self.to_u8() as usize]
    }
    pub fn knight_moves(&self) -> PossibleMoves {
        knight_moves[self.to_u8() as usize]
    }
}

impl State {
    pub fn is_mate(&self) -> bool {
        ((self.black_king.king_moves() | PossibleMoves::from_position(self.black_king)) / self.covered_by_white()) == PossibleMoves::empty()
    }

    pub fn black_in_check(&self) -> bool {
        self.covered_by_white().contains(self.black_king)
    }

    pub fn covered_by_white(&self) -> PossibleMoves {
        self.white_king.king_moves() | self.knights[0].knight_moves() | self.knights[1].knight_moves() | self.knights[2].knight_moves()
    }

    pub fn white_knights(&self) -> PossibleMoves {
        PossibleMoves::from_position(self.knights[0]) | PossibleMoves::from_position(self.knights[1]) | PossibleMoves::from_position(self.knights[2])
    }

    pub fn white_pieces(&self) -> PossibleMoves {
        PossibleMoves::from_position(self.white_king) | self.white_knights()
    }

    pub fn pieces(&self) -> PossibleMoves {
        self.white_pieces() | PossibleMoves::from_position(self.black_king)
    }

    pub fn next_states(&self) -> Vec<Self> {
        let mut result = Vec::new();

        if self.white_to_move {
            for pos in (self.white_king.king_moves() / (self.pieces() | self.black_king.king_moves())).iter() {
                result.push(State {
                    white_king: pos,
                    white_to_move: false,
                    ..*self
                })
            }
            for pos in (self.knights[0].knight_moves() / (self.pieces())).iter() {
                result.push(State {
                    knights: [pos, self.knights[1], self.knights[2]],
                    white_to_move: false,
                    ..*self
                })
            }
            for pos in (self.knights[1].knight_moves() / (self.pieces())).iter() {
                result.push(State {
                    knights: [self.knights[0], pos, self.knights[2]],
                    white_to_move: false,
                    ..*self
                })
            }
            for pos in (self.knights[2].knight_moves() / (self.pieces())).iter() {
                result.push(State {
                    knights: [self.knights[0], self.knights[1], pos],
                    white_to_move: false,
                    ..*self
                })
            }
        }
        else {
            for pos in (self.black_king.king_moves() / (self.white_pieces() | self.covered_by_white())).iter() {
                result.push(State {
                    black_king: pos,
                    white_to_move: true,
                    ..*self
                })
            }
        }

        result
    }

    pub fn next_states_count(&self) -> u8 {
        if self.white_to_move {
            let mut res = 0;
            res += (self.white_king.king_moves() / (self.pieces() | self.black_king.king_moves())).count();
            res += (self.knights[0].knight_moves() / (self.pieces())).count();
            res += (self.knights[1].knight_moves() / (self.pieces())).count();
            res += (self.knights[2].knight_moves() / (self.pieces())).count();
            res
        }
        else {
            (self.black_king.king_moves() / (self.white_pieces() | self.covered_by_white())).count()
        }
    }

    pub fn previous_states(&self) -> Vec<Self> {
        let mut result = Vec::new();

        if self.white_to_move {
            assert_eq!(self.black_in_check(), false);
            for pos in (self.black_king.king_moves() / (self.pieces() | self.white_king.king_moves())).iter() {
                result.push(State {
                    black_king: pos,
                    white_to_move: false,
                    ..*self
                })
            }
        }
        else {
            for pos in (self.white_king.king_moves() / (self.pieces() | self.black_king.king_moves())).iter() {
                let state = State {
                    white_king: pos,
                    white_to_move: true,
                    ..*self
                };
                if !state.black_in_check() {
                    result.push(state)
                }
            }
            for pos in (self.knights[0].knight_moves() / self.pieces()).iter() {
                let state = State {
                    knights: [pos, self.knights[1], self.knights[2]],
                    white_to_move: true,
                    ..*self
                };
                if !state.black_in_check() {
                    result.push(state)
                }
            }
            for pos in (self.knights[1].knight_moves() / self.pieces()).iter() {
                let state = State {
                    knights: [self.knights[0], pos, self.knights[2]],
                    white_to_move: true,
                    ..*self
                };
                if !state.black_in_check() {
                    result.push(state)
                }
            }
            for pos in (self.knights[2].knight_moves() / self.pieces()).iter() {
                let state = State {
                    knights: [self.knights[0], self.knights[1], pos],
                    white_to_move: true,
                    ..*self
                };
                if !state.black_in_check() {
                    result.push(state)
                }
            }
        }

        result
    }
}
