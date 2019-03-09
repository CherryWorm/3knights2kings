pub type PackedState = u64;

#[derive(Debug, Clone, Copy)]
pub struct SmallState {
    // < 16
    pub white_king: u8,
    // sorted and < 60
    pub knights: [u8; 3],
    // < 63
    pub black_king: u8,
    // < 28
    pub target_field: u8,
    // < 2
    pub white_to_move: u8,
}

pub fn to_knight_index(knights: [u8; 3]) -> usize {
    knights[0] as usize + 60 * (knights[1] as usize + 60 * knights[2] as usize)
}

lazy_static! {
    pub static ref knight_tables: (Vec<u32>, Vec<[u8; 3]>) = {
        let mut table1 = Vec::new();
        table1.resize(60 * 60 * 60, 0);
        let mut table2 = Vec::new();

        for knight3 in 0..60 {
            for knight2 in 0..=knight3 {
                for knight1 in 0..=knight2 {
                    let knights = [knight1 as u8, knight2 as u8, knight3 as u8];
                    table1[to_knight_index([knight1, knight2, knight3])] = table2.len() as u32;
                    table2.push(knights);
                }
            }
        }
        // println!("{}", table2.len()); -- 37820
        (table1, table2)
    };
}

impl SmallState {
    pub fn encode(&self) -> PackedState {
        assert!(self.white_king < 16);
        assert!(self.black_king < 63);
        for knight in &self.knights {
            assert!(*knight < 60);
        }
        assert!(self.target_field < 28);
        assert!(self.white_to_move < 2);
        self.white_king as u64 + 16 * (self.black_king as u64 + 63 * (knight_tables.0[to_knight_index(self.knights)] as u64 + knight_tables.1.len() as u64 * (self.target_field as u64 + 28 * self.white_to_move as u64)))
    }
    pub fn decode(mut packed: PackedState) -> Self {
        let white_king = (packed % 16) as u8;
        packed /= 16;
        let black_king = (packed % 63) as u8;
        packed /= 63;
        let knights = knight_tables.1[packed as usize % knight_tables.1.len()];
        packed /= knight_tables.1.len() as u64;
        let target_field = (packed % 28) as u8;
        packed /= 28;
        let white_to_move = packed as u8;
        SmallState {
            white_king,
            black_king,
            target_field,
            knights,
            white_to_move,
        }
    }
}
