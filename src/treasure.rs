use std::fmt::Display;
use strum_macros::EnumCount;

use rand::Rng;

#[derive(Clone, Copy, Debug, EnumCount, PartialEq, Eq, Hash)]
pub enum Treasure {
    One,
    Two,
    Three,
    Four,
}
pub const MAX_NUM_TREASURES: usize = 10;

impl Treasure {
    fn idx(&self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }

    fn base_value(&self) -> u32 {
        self.idx() as u32 * 4
    }
}

impl Display for Treasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "."),
            Self::Two => write!(f, ":"),
            Self::Three => write!(f, "%"),
            Self::Four => write!(f, "#"),
        }
    }
}

pub struct TreasureValueAssigner {
    buckets: [[u32; 4]; 4],
}

impl TreasureValueAssigner {
    pub fn new() -> Self {
        Self {
            buckets: [[2; 4]; 4],
        }
    }

    pub fn assign_value(&mut self, treasure: Treasure) -> u32 {
        let idx = treasure.idx();
        let bucket = &mut self.buckets[idx];
        debug_assert!(bucket.iter().any(|&count| count > 0));

        let total: u32 = bucket.iter().sum();
        let mut choice = rand::rng().random_range(1..=total);
        let (value_idx, count) = bucket
            .iter_mut()
            .enumerate()
            .find(|&(_, &mut count)| {
                choice = choice.saturating_sub(count);
                choice == 0
            })
            .unwrap();
        *count -= 1;

        treasure.base_value() + value_idx as u32
    }
}

impl Default for TreasureValueAssigner {
    fn default() -> Self {
        Self::new()
    }
}
