use std::fmt::Display;

use bit_set::BitSet;
use itertools::Itertools;
use lazy_static::lazy_static;
use strum_macros::EnumCount;
use termion::color;

use crate::{
    error::{DeepSeaError, DeepSeaResult},
    solver::TreasureDecision,
    treasure::Treasure,
};

#[derive(Clone, Copy, Debug, EnumCount, PartialEq, Eq, Hash)]
pub enum Tile {
    Empty,
    Treasure(Treasure),
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "_"),
            Self::Treasure(treasure) => write!(f, "{treasure}"),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumCount, PartialEq, Eq, Hash)]
pub enum DiveDirection {
    Down,
    Up,
}

#[derive(Clone, Copy, Debug, EnumCount, PartialEq, Eq, Hash)]
pub enum Position {
    /// Index into `DeepSea::path`.
    Diving(usize),
    WaitingToDive,
    ReturnedToSubmarine,
}

impl Position {
    pub fn as_diving(&self) -> Option<usize> {
        if let Self::Diving(index) = self {
            Some(*index)
        } else {
            None
        }
    }

    pub fn advance(&self, direction: DiveDirection) -> DeepSeaResult<Position> {
        match (self, direction) {
            (Self::Diving(0), DiveDirection::Up) => Ok(Self::ReturnedToSubmarine),
            (Self::Diving(index), DiveDirection::Up) => Ok(Self::Diving(index - 1)),
            (Self::Diving(index), DiveDirection::Down) => Ok(Self::Diving(index + 1)),
            (Self::WaitingToDive, DiveDirection::Down) => Ok(Self::Diving(0)),
            (Self::WaitingToDive, DiveDirection::Up) => Err(DeepSeaError::Internal(
                "Cannot move up before leaving submarine".to_owned(),
            )
            .into()),
            (Self::ReturnedToSubmarine, _) => Err(DeepSeaError::Internal(
                "Cannot move after returned to submarine".to_owned(),
            )
            .into()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    direction: DiveDirection,
    position: Position,
    held_treasures: Vec<Treasure>,
}

impl Player {
    fn new() -> Self {
        Self {
            direction: DiveDirection::Down,
            position: Position::WaitingToDive,
            held_treasures: vec![],
        }
    }

    pub fn direction(&self) -> DiveDirection {
        self.direction
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn held_treasures(&self) -> &[Treasure] {
        &self.held_treasures
    }
}

#[derive(Clone, Debug)]
pub struct DeepSea {
    path: Vec<Tile>,
    players: Vec<Player>,
    player_idx: usize,
    oxygen: u32,
    occupied_tiles: BitSet,
}

impl DeepSea {
    pub const OXYGEN: u32 = 25;

    pub fn new(path: Vec<Tile>, num_players: usize) -> Self {
        let path_len = path.len();
        Self {
            path,
            players: (0..num_players).map(|_| Player::new()).collect(),
            player_idx: 0,
            oxygen: Self::OXYGEN,
            occupied_tiles: BitSet::with_capacity(path_len),
        }
    }

    pub fn path(&self) -> &[Tile] {
        &self.path
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn player_idx(&self) -> usize {
        self.player_idx
    }

    pub fn oxygen(&self) -> u32 {
        self.oxygen
    }

    pub fn occupied_tiles(&self) -> &BitSet {
        &self.occupied_tiles
    }

    fn at_end(&self, position: Position) -> bool {
        position == Position::Diving(self.path.len() - 1)
    }

    fn occupied(&self, position: Position) -> bool {
        match position {
            Position::Diving(index) => self.occupied_tiles.contains(index),
            Position::ReturnedToSubmarine | Position::WaitingToDive => false,
        }
    }

    fn leave_tile(&mut self, position: Position) {
        match position {
            Position::Diving(index) => {
                self.occupied_tiles.remove(index);
            }
            Position::WaitingToDive | Position::ReturnedToSubmarine => {}
        }
    }

    fn enter_tile(&mut self, position: Position) {
        match position {
            Position::Diving(index) => {
                self.occupied_tiles.insert(index);
            }
            Position::WaitingToDive | Position::ReturnedToSubmarine => {}
        }
    }

    pub fn done(&self) -> bool {
        self.oxygen == 0
            || self
                .players
                .iter()
                .all(|player| player.position() == Position::ReturnedToSubmarine)
    }

    pub fn take_oxygen(&mut self) {
        let player = &self.players[self.player_idx];
        self.oxygen = self
            .oxygen
            .saturating_sub(player.held_treasures.len() as u32);
    }

    pub fn move_player(&mut self, direction: DiveDirection, mut dice_roll: u32) -> DeepSeaResult {
        let player = &self.players[self.player_idx];
        dice_roll = dice_roll.saturating_sub(player.held_treasures().len() as u32);

        let mut cur_player_pos = player.position();
        let mut player_pos = cur_player_pos;
        while dice_roll > 0
            && cur_player_pos != Position::ReturnedToSubmarine
            && cur_player_pos != Position::Diving(self.path.len() - 1)
        {
            if direction == DiveDirection::Down && self.at_end(player_pos) {
                break;
            }

            cur_player_pos = cur_player_pos.advance(direction)?;
            if !self.occupied(cur_player_pos) {
                dice_roll -= 1;
                player_pos = cur_player_pos;
            }
        }

        self.leave_tile(player.position());
        self.enter_tile(player_pos);
        let player = &mut self.players[self.player_idx];
        player.direction = direction;
        player.position = player_pos;
        Ok(())
    }

    pub fn take_treasure(&mut self, treasure: TreasureDecision) -> DeepSeaResult {
        let player = &mut self.players[self.player_idx];
        let tile_idx = player.position().as_diving().unwrap();
        match treasure {
            TreasureDecision::Take => {
                if let Tile::Treasure(treasure) = self.path[tile_idx] {
                    player.held_treasures.push(treasure);
                    self.path[tile_idx] = Tile::Empty;
                    Ok(())
                } else {
                    Err(DeepSeaError::AgentError(format!(
                        "Cannot take treasure from {tile_idx}: {:?}",
                        self.path[tile_idx]
                    ))
                    .into())
                }
            }
            TreasureDecision::Return(treasure) => {
                if self.path[tile_idx] != Tile::Empty {
                    Err(DeepSeaError::AgentError(format!(
                        "Cannot put treasure back in non-empty tile {tile_idx}: {:?}",
                        self.path[tile_idx]
                    ))
                    .into())
                } else if let Some((treasure_idx, &treasure)) = player
                    .held_treasures
                    .iter()
                    .find_position(|t| **t == treasure)
                {
                    player.held_treasures.remove(treasure_idx);
                    self.path[tile_idx] = Tile::Treasure(treasure);
                    Ok(())
                } else {
                    Err(DeepSeaError::AgentError(format!(
                        "Player is not holding treasure {:?}: {:?}",
                        treasure, player.held_treasures
                    ))
                    .into())
                }
            }
            TreasureDecision::Ignore => Ok(()),
        }
    }

    pub fn next_player(&mut self) {
        self.player_idx = (self.player_idx + 1) % self.players.len();
    }
}

impl Display for DeepSea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        lazy_static! {
            static ref PLAYER_COLORS: [Box<dyn color::Color + Sync>; 6] = [
                Box::new(color::Red),
                Box::new(color::Green),
                Box::new(color::Blue),
                Box::new(color::Magenta),
                Box::new(color::Rgb(0xff, 0xf4, 0x4f)),
                Box::new(color::Rgb(0xff, 0x99, 0x1c)),
            ];
        }

        writeln!(f, "Oxygen: {}", self.oxygen)?;
        for player_idx in 0..self.players.len() {
            PLAYER_COLORS[player_idx % PLAYER_COLORS.len()].write_fg(f)?;
            write!(f, "{player_idx}{}: ", color::Fg(color::Reset))?;
            for treasure in &self.players[player_idx].held_treasures {
                write!(f, "{treasure}")?;
            }
            writeln!(f)?;
        }

        for idx in (0..self.path.len()).rev() {
            let position = Position::Diving(idx);
            if self.occupied(position) {
                let (player_idx, player) = self
                    .players
                    .iter()
                    .find_position(|player| player.position == position)
                    .unwrap();
                PLAYER_COLORS[player_idx % PLAYER_COLORS.len()].write_fg(f)?;
                write!(
                    f,
                    "{}{}",
                    match player.direction {
                        DiveDirection::Down => '<',
                        DiveDirection::Up => '>',
                    },
                    color::Fg(color::Reset)
                )?;
            } else {
                write!(f, " ")?;
            }
        }
        writeln!(f)?;
        for tile in self.path.iter().rev() {
            write!(f, "{tile}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use googletest::{
        expect_eq, expect_false, expect_that, expect_true, gtest,
        prelude::{empty, ok, pat, unordered_elements_are},
    };

    use crate::{
        deep_sea::{DeepSea, DiveDirection, Player, Position, Tile, Treasure},
        error::DeepSeaResult,
        solver::TreasureDecision,
    };

    #[gtest]
    fn test_advance_position() {
        expect_that!(
            Position::WaitingToDive.advance(DiveDirection::Down),
            ok(pat!(Position::Diving(&0)))
        );
        expect_true!(Position::WaitingToDive.advance(DiveDirection::Up).is_err());
        expect_true!(
            Position::ReturnedToSubmarine
                .advance(DiveDirection::Down)
                .is_err()
        );
        expect_true!(
            Position::ReturnedToSubmarine
                .advance(DiveDirection::Up)
                .is_err()
        );

        expect_that!(
            Position::Diving(10).advance(DiveDirection::Down),
            ok(pat!(Position::Diving(&11)))
        );
        expect_that!(
            Position::Diving(8).advance(DiveDirection::Up),
            ok(pat!(Position::Diving(&7)))
        );
        expect_that!(
            Position::Diving(0).advance(DiveDirection::Up),
            ok(pat!(Position::ReturnedToSubmarine))
        );
    }

    #[gtest]
    fn test_init() -> DeepSeaResult {
        let deep_sea = DeepSea::new(vec![Tile::Empty], 1);

        expect_that!(
            deep_sea.players[0],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::WaitingToDive),
                held_treasures: empty(),
            })
        );
        expect_false!(deep_sea.occupied(Position::Diving(0)));

        Ok(())
    }

    #[gtest]
    fn test_move_player() -> DeepSeaResult {
        let mut deep_sea = DeepSea::new(vec![Tile::Empty], 1);

        deep_sea.move_player(DiveDirection::Down, 1)?;
        expect_that!(
            deep_sea.players[0],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::Diving(&0)),
                held_treasures: empty(),
            })
        );
        expect_true!(deep_sea.occupied(Position::Diving(0)));

        Ok(())
    }

    #[gtest]
    fn test_leapfrog_player() -> DeepSeaResult {
        let mut deep_sea = DeepSea::new((0..3).map(|_| Tile::Empty).collect(), 2);

        deep_sea.move_player(DiveDirection::Down, 2)?;
        expect_that!(
            deep_sea.players[0],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::Diving(&1)),
                held_treasures: empty(),
            })
        );
        expect_true!(deep_sea.occupied(Position::Diving(1)));

        deep_sea.next_player();

        deep_sea.move_player(DiveDirection::Down, 2)?;
        expect_that!(
            deep_sea.players[1],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::Diving(&2)),
                held_treasures: empty(),
            })
        );
        expect_true!(deep_sea.occupied(Position::Diving(2)));

        Ok(())
    }

    #[gtest]
    fn test_move_player_twice() -> DeepSeaResult {
        let mut deep_sea = DeepSea::new((0..3).map(|_| Tile::Empty).collect(), 1);

        deep_sea.move_player(DiveDirection::Down, 2)?;
        expect_that!(
            deep_sea.players[0],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::Diving(&1)),
                held_treasures: empty(),
            })
        );
        expect_true!(deep_sea.occupied(Position::Diving(1)));

        deep_sea.next_player();

        deep_sea.move_player(DiveDirection::Down, 1)?;
        expect_that!(
            deep_sea.players[0],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::Diving(&2)),
                held_treasures: empty(),
            })
        );
        expect_false!(deep_sea.occupied(Position::Diving(1)));
        expect_true!(deep_sea.occupied(Position::Diving(2)));

        Ok(())
    }

    #[gtest]
    fn test_take_treasure() -> DeepSeaResult {
        let mut deep_sea = DeepSea::new(vec![Tile::Treasure(Treasure::One)], 1);

        deep_sea.move_player(DiveDirection::Down, 1)?;
        expect_that!(
            deep_sea.players[0],
            pat!(Player {
                direction: pat!(DiveDirection::Down),
                position: pat!(Position::Diving(&0)),
                held_treasures: empty(),
            })
        );
        deep_sea.take_treasure(TreasureDecision::Ignore)?;
        expect_eq!(deep_sea.path[0], Tile::Treasure(Treasure::One));
        expect_that!(deep_sea.players[0].held_treasures, empty());

        deep_sea.take_treasure(TreasureDecision::Take)?;
        expect_eq!(deep_sea.path[0], Tile::Empty);
        expect_that!(
            deep_sea.players[0].held_treasures,
            unordered_elements_are![&Treasure::One]
        );

        expect_true!(deep_sea.take_treasure(TreasureDecision::Take).is_err());

        expect_true!(
            deep_sea
                .take_treasure(TreasureDecision::Return(Treasure::Four))
                .is_err()
        );
        deep_sea.take_treasure(TreasureDecision::Return(Treasure::One))?;
        expect_eq!(deep_sea.path[0], Tile::Treasure(Treasure::One));
        expect_that!(deep_sea.players[0].held_treasures, empty());

        Ok(())
    }
}
