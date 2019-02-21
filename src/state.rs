use crate::encoding::{PackedState, SmallState};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn from_u8(i: u8) -> Self {
        Position { x: i & 0b111, y: (i >> 3) & 0b111 }
    }
    pub fn to_u8(&self) -> u8 {
        self.x | (self.y << 3)
    }

    pub fn from_u8_rim(i: u8) -> Self {
        assert!(i < 28);
        if i < 8 {
            Position { x: i, y: 0 }
        } else if i < 16 {
            Position { x: i - 8, y: 7 }
        } else if i < 22 {
            Position { x: 0, y: i - 15 }
        } else {
            Position { x: 7, y: i - 21 }
        }
    }
    pub fn to_u8_rim(self) -> u8 {
        assert!(self.x == 0 || self.y == 0 || self.x == 7 || self.y == 7);
        if self.y == 0 {
            self.x
        } else if self.y == 7 {
            self.x + 8
        } else if self.x == 0 {
            self.y + 15
        } else {
            self.y + 21
        }
    }

    pub fn from_u8_bottom_left(i: u8) -> Self {
        assert!(i < 16);
        Position { x: i & 0b11, y: (i >> 2) & 0b11 }
    }
    pub fn to_u8_bottom_left(&self) -> u8 {
        assert!(self.x < 4 && self.y < 4);
        self.x | (self.y << 2)
    }

    pub fn rotate_clockwise(&self) -> Self {
        Position { x: self.y, y: 7 - self.x }
    }
    pub fn rotate_counterclockwise(&self) -> Self {
        Position { x: 7 - self.y, y: self.x }
    }
    pub fn rotate_twice(&self) -> Self {
        Position { x: 7 - self.x, y: 7 - self.y }
    }

    pub fn is_out_of_bounds(&self, dx: i16, dy: i16) -> bool {
        self.x as i16 + dx > 7 || self.x as i16 + dx < 0 || self.y as i16 + dy > 7 || self.y as i16 + dy < 0
    }

    pub fn add(&self, dx: i16, dy: i16) -> Self {
        Position {
            x: (self.x as i16 + dx) as u8,
            y: (self.y as i16 + dy) as u8,
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Position) -> Ordering {
        self.to_u8().cmp(&(*other).to_u8())
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn sort<A, T>(mut array: A) -> A
where
    A: AsMut<[T]>,
    T: Ord,
{
    let slice = array.as_mut();
    slice.sort();
    array
}

#[derive(Debug, Clone, Copy)]
pub struct State {
    pub white_king: Position,
    pub knights: [Position; 3],
    pub black_king: Position,
    pub target_field: Position,
    pub white_to_move: bool,
}

impl State {
    pub fn unpack(packed: PackedState) -> Self {
        let SmallState {
            white_king: white_king_packed,
            knights: knights_packed,
            black_king: black_king_packed,
            target_field: target_field_packed,
            white_to_move: white_to_move_packed,
        } = SmallState::decode(packed);

        debug!("{:?}", SmallState::decode(packed));

        let white_king = Position::from_u8_bottom_left(white_king_packed);
        let black_king = black_king_packed + if white_king.to_u8() <= black_king_packed { 1 } else { 0 };
        let mut knights = [0u8; 3];

        let mut prev: Vec<u8> = vec![white_king.to_u8(), black_king];
        for i in 0..3 {
            knights[i] = knights_packed[i];
            prev.sort_unstable();
            for j in &prev {
                debug!("knights[{}] = {}; {}", i, knights[i], *j);
                if *j <= knights[i] {
                    knights[i] += 1;
                }
            }
            prev.push(knights[i]);
        }

        let black_king = Position::from_u8(black_king);
        let knights = [Position::from_u8(knights[0]), Position::from_u8(knights[1]), Position::from_u8(knights[2])];
        let target_field = Position::from_u8_rim(target_field_packed);
        let white_to_move = white_to_move_packed == 1;

        State {
            white_king,
            knights,
            black_king,
            white_to_move,
            target_field,
        }
    }

    pub fn normalize(&self) -> State {
        let (white_king, knights, black_king, target_field) = if self.white_king.x >= 4 && self.white_king.y < 4 {
            // lower right
            (
                self.white_king.rotate_clockwise(),
                [self.knights[0].rotate_clockwise(), self.knights[1].rotate_clockwise(), self.knights[2].rotate_clockwise()],
                self.black_king.rotate_clockwise(),
                self.target_field.rotate_clockwise(),
            )
        } else if self.white_king.x < 4 && self.white_king.y >= 4 {
            // upper left
            (
                self.white_king.rotate_counterclockwise(),
                [self.knights[0].rotate_counterclockwise(), self.knights[1].rotate_counterclockwise(), self.knights[2].rotate_counterclockwise()],
                self.black_king.rotate_counterclockwise(),
                self.target_field.rotate_counterclockwise(),
            )
        } else if self.white_king.x >= 4 && self.white_king.y >= 4 {
            // upper right
            (
                self.white_king.rotate_twice(),
                [self.knights[0].rotate_twice(), self.knights[1].rotate_twice(), self.knights[2].rotate_twice()],
                self.black_king.rotate_twice(),
                self.target_field.rotate_twice(),
            )
        } else {
            // lower left
            (self.white_king, [self.knights[0], self.knights[1], self.knights[2]], self.black_king, self.target_field)
        };

        let knights = sort(knights);

        State { white_king, black_king, knights, target_field, ..*self }
    }

    pub fn pack(&self) -> PackedState {
        let State {
            white_king,
            black_king,
            knights,
            white_to_move,
            target_field,
        } = self.normalize();

        let mut prev: Vec<u8> = vec![white_king.to_u8(), black_king.to_u8()];
        let black_king = black_king.to_u8() - if white_king < black_king { 1 } else { 0 };

        let mut knights_packed = [0u8; 3];

        for i in 0..3 {
            knights_packed[i] = knights[i].to_u8();
            prev.sort_unstable();
            prev.reverse();
            for j in &prev {
                debug!("knights[{}] = {}; {}", i, knights[i].to_u8(), *j);
                if *j <= knights[i].to_u8() {
                    knights_packed[i] -= 1;
                }
            }
            prev.push(knights[i].to_u8());
        }

        let white_to_move = if self.white_to_move { 1u8 } else { 0u8 };

        debug!(
            "{:?}",
            SmallState {
                white_king: white_king.to_u8_bottom_left(),
                black_king,
                knights: knights_packed,
                target_field: target_field.to_u8_rim(),
                white_to_move,
            }
        );

        SmallState {
            white_king: white_king.to_u8_bottom_left(),
            black_king,
            knights: knights_packed,
            target_field: target_field.to_u8_rim(),
            white_to_move,
        }
        .encode()
    }

    pub fn to_fen(self) -> String {
        let mut result = String::from("");
        let mut position = [""; 64];
        position[self.white_king.to_u8() as usize] = "K";
        position[self.black_king.to_u8() as usize] = "k";
        for knight in &self.knights {
            position[knight.to_u8() as usize] = "N";
        }
        let mut counter = 0;
        for i in 0..8 {
            for j in 0..8 {
                if position[i * 8 + j] == "" {
                    counter += 1;
                } else {
                    if counter > 0 {
                        result += &counter.to_string();
                        counter = 0;
                    }
                    result += position[i * 8 + j];
                }
            }
            if counter != 0 {
                result += &counter.to_string();
                counter = 0;
            }
            result += "/";
        }
        result + " " + if self.white_to_move { "w" } else { "b" } + " ---- - 0 1"
    }

    pub fn to_lichess(self) -> String {
        String::from("https://lichess.org/editor/") + &self.to_fen().replace(" ", "_")
    }

    pub fn next_moves(self) -> Vec<State> {
        if self.white_to_move {

        } else {

        }
        unimplemented!()
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        let normalized = self.normalize();
        let o_normalized = other.normalize();
        (normalized.white_king, normalized.black_king, normalized.knights, normalized.target_field, normalized.white_to_move) == (o_normalized.white_king, o_normalized.black_king, o_normalized.knights, o_normalized.target_field, o_normalized.white_to_move)
    }
}
