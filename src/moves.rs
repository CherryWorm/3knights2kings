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

    pub fn contains(&self, pos: Position) -> bool {
        (self.0 >> pos.to_u8()) & 1 == 1
    }

    fn from_position(pos: Position) -> Self {
        PossibleMoves(1 << pos.to_u8())
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

impl ops::Div<PossibleMoves> for PossibleMoves {
    type Output = PossibleMoves;
    fn div(self, other: PossibleMoves) -> PossibleMoves {
        self.dif(other)
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
                    m = add_if_not_out_of_bounds(m, pos, dx, dy);
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
        (self.black_king.king_moves() | PossibleMoves::from_position(self.black_king)) / (self.white_king.king_moves() | self.knights[0].knight_moves() | self.knights[1].knight_moves() | self.knights[2].knight_moves()) == PossibleMoves::empty()
    }
}
