use crate::{
    deep_sea,
    deep_sea::*,
    ml::vectorization::*,
    solver::TreasureDecision,
    treasure::{MAX_NUM_TREASURES, Treasure},
};
use bit_set::BitSet;
use itertools::{self, Itertools};
use strum::EnumCount;

pub const TREASURE_DECISION_COUNT: usize = 2 + Treasure::COUNT;
pub const POSITION_COUNT: usize = Position::COUNT;

pub struct Path {
    pub tiles: Vec<Tile>,
    pub occupied: BitSet,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub direction: DiveDirection,
    pub position: Position,
    pub held_treasures: Vec<Treasure>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            direction: DiveDirection::Down,
            position: Position::WaitingToDive,
            held_treasures: vec![],
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl From<deep_sea::Player> for Player {
    fn from(ds_player: deep_sea::Player) -> Self {
        Self {
            direction: ds_player.direction(),
            position: ds_player.position(),
            held_treasures: ds_player.held_treasures().to_vec().clone(),
        }
    }
}

pub struct DeepSeaState {
    pub path: Path,
    pub players: Vec<Player>,
    pub oxygen: u16,
}

impl Default for DeepSeaState {
    fn default() -> Self {
        let tiles: Vec<Tile> = (0..8)
            .map(|_| Tile::Treasure(Treasure::One))
            .chain((0..8).map(|_| Tile::Treasure(Treasure::Two)))
            .chain((0..8).map(|_| Tile::Treasure(Treasure::Three)))
            .chain((0..8).map(|_| Tile::Treasure(Treasure::Four)))
            .collect();
        let path = Path {
            tiles,
            occupied: BitSet::new(),
        };
        Self {
            path,
            players: vec![Player::new(); 6],
            oxygen: DeepSea::OXYGEN as u16,
        }
    }
}

pub enum DeepSeaAction {
    TreasureDecision(TreasureDecision),
    DiveDirection(DiveDirection),
}

impl From<TreasureDecision> for DeepSeaAction {
    fn from(td: TreasureDecision) -> Self {
        Self::TreasureDecision(td)
    }
}

impl From<DiveDirection> for DeepSeaAction {
    fn from(dd: DiveDirection) -> Self {
        Self::DiveDirection(dd)
    }
}

pub struct DeepSeaStateActionPair<'a> {
    pub state: &'a DeepSeaState,
    pub action: &'a DeepSeaAction,
}

pub const DEEP_SEA_ACTION_COUNT: usize = TREASURE_DECISION_COUNT + DiveDirection::COUNT;

impl Unpackable for Treasure {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        // 1-indexed because it's a treasure value.
        [T::from(1 + *self as u16)].into_iter()
    }

    fn unpacked_size(&self) -> usize {
        1
    }
}

impl Unpackable for DiveDirection {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        [T::from(*self as u16)].into_iter()
    }

    fn unpacked_size(&self) -> usize {
        1
    }
}

impl Unpackable for Position {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        let position_val: u16 = match self {
            Position::Diving(depth) => *depth as u16,
            _ => 0,
        };
        [
            T::from(position_val),
            T::from(*self == Position::WaitingToDive),
            T::from(*self == Position::ReturnedToSubmarine),
        ]
        .into_iter()
    }

    fn unpacked_size(&self) -> usize {
        POSITION_COUNT
    }
}

impl Unpackable for Tile {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        match self {
            Tile::Empty => UnifiedIterator::Opt1([T::zero()].into_iter()),
            Tile::Treasure(treasure) => UnifiedIterator::Opt2(treasure.unpack::<T>()),
        }
    }

    fn unpacked_size(&self) -> usize {
        1
    }
}

impl Unpackable for Path {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        let path_iter = self.tiles.unpack::<T>();
        let occupied_iter = (0..self.tiles.len()).map(|idx| T::from(self.occupied.contains(idx)));
        itertools::interleave(path_iter, occupied_iter)
    }

    fn unpacked_size(&self) -> usize {
        2 * self.tiles.len()
    }
}

impl Unpackable for Player {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        let direction = self.direction.unpack::<T>();
        let position = self.position.unpack::<T>();
        let held_treasures = self.held_treasures.unpack::<T>();
        let filled_size = self.direction.unpacked_size()
            + self.position.unpacked_size()
            + self.held_treasures.unpacked_size();
        direction
            .chain(position)
            .chain(held_treasures)
            .chain(std::iter::repeat_n(
                T::zero(),
                self.unpacked_size().saturating_sub(filled_size),
            ))
    }

    fn unpacked_size(&self) -> usize {
        self.direction.unpacked_size() + self.position.unpacked_size() + MAX_NUM_TREASURES
    }
}

impl Unpackable for DeepSeaAction {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        // let mut unpacked = vec![T::zero(); DEEP_SEA_ACTION_ENUM_COUNT];
        let hot_idx = match self {
            DeepSeaAction::TreasureDecision(td) => match td {
                TreasureDecision::Ignore => 0,
                TreasureDecision::Take => 1,
                TreasureDecision::Return(t) => 2 + *t as usize,
            },
            DeepSeaAction::DiveDirection(dd) => TREASURE_DECISION_COUNT + *dd as usize,
        };
        std::iter::repeat_n(T::zero(), hot_idx)
            .chain(std::iter::once(T::from(1u16)))
            .chain(std::iter::repeat_n(
                T::zero(),
                self.unpacked_size() - hot_idx - 1,
            ))
    }

    fn unpacked_size(&self) -> usize {
        DEEP_SEA_ACTION_COUNT
    }
}

impl Unpackable for DeepSeaState {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        self.path
            .unpack::<T>()
            .collect_vec()
            .into_iter()
            .chain(self.players.unpack())
            .chain([T::from(self.oxygen)])
    }

    fn unpacked_size(&self) -> usize {
        let path_dim = self.path.unpacked_size();
        let players_dim = self.players.unpacked_size();
        let oxygen_dim = 1;
        path_dim + players_dim + oxygen_dim
    }
}

impl<'a> Unpackable for DeepSeaStateActionPair<'a> {
    fn unpack<T: DataType>(&self) -> impl Iterator<Item = T> {
        self.state.unpack::<T>().chain(self.action.unpack::<T>())
    }

    fn unpacked_size(&self) -> usize {
        self.state.unpacked_size() + self.action.unpacked_size()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        deep_sea::{DeepSea, DiveDirection, Position, Tile},
        deep_sea_vectorization::{
            DEEP_SEA_ACTION_COUNT, DeepSeaAction, DeepSeaState, DeepSeaStateActionPair, Path,
            Player,
        },
        ml::vectorization::*,
        solver::TreasureDecision,
        treasure::Treasure,
    };
    use bit_set::BitSet;

    #[test]
    fn test_tiles_vectorization() {
        let tiles: Vec<Tile> = (0..8)
            .map(|_| Tile::Treasure(Treasure::One))
            .chain((0..8).map(|_| Tile::Treasure(Treasure::Two)))
            .chain((0..8).map(|_| Tile::Treasure(Treasure::Three)))
            .chain((0..8).map(|_| Tile::Treasure(Treasure::Four)))
            .collect();
        let tiles_size = &[tiles.unpacked_size()];
        let tiles_f32_ndarray: ndarray::Array1<f32> = tiles.into_ndarray();
        assert_eq!(tiles_f32_ndarray.shape(), tiles_size);
    }

    #[test]
    fn test_path_vectorization() {
        let tiles: Vec<Tile> = (0..1)
            .map(|_| Tile::Treasure(Treasure::One))
            .chain((0..1).map(|_| Tile::Treasure(Treasure::Two)))
            .chain((0..1).map(|_| Tile::Treasure(Treasure::Three)))
            .chain((0..1).map(|_| Tile::Treasure(Treasure::Four)))
            .collect();
        let mut occupied_tiles: BitSet = BitSet::new();
        let occupied_idx = 3;
        occupied_tiles.insert(occupied_idx);
        let path: Path = Path {
            tiles: tiles.clone(),
            occupied: occupied_tiles.clone(),
        };
        let path_size = &[path.unpacked_size()];
        let path_f64_ndarray: ndarray::Array1<f64> = path.into_ndarray();
        assert_eq!(path_f64_ndarray.shape(), path_size);
        for (i, tile) in tiles.iter().enumerate() {
            assert_eq!(
                path_f64_ndarray[[2 * i]],
                tile.unpack::<f64>().next().unwrap()
            );
            assert_eq!(
                path_f64_ndarray[[2 * i + 1]],
                f64::from(occupied_tiles.contains(i))
            );
        }
    }

    #[test]
    fn test_direction_vectorization() {
        let direction = DiveDirection::Up;
        let direction_size = &[direction.unpacked_size()];
        let direction_f32_ndarray: ndarray::Array1<f32> = direction.into_ndarray();
        assert_eq!(direction_f32_ndarray.shape(), direction_size);
        assert_eq!(
            direction_f32_ndarray[[0]],
            f32::from(DiveDirection::Up as u8)
        )
    }

    #[test]
    fn test_position_vectorization() {
        let position = Position::Diving(40);
        let position_size = &[position.unpacked_size()];
        let position_f64_ndarray = position.into_ndarray::<f64>();
        assert_eq!(position_f64_ndarray.shape(), position_size);
    }

    #[test]
    fn test_player_vectorization() {
        // A hack to get Player since ::new() is private.
        let deep_sea = DeepSea::new(vec![], 1);
        let player = Player::from(deep_sea.players()[0].clone());
        let player_size = &[player.unpacked_size()];
        let player_i32_ndarray: ndarray::Array1<i32> = player.clone().into_ndarray();
        assert_eq!(player_i32_ndarray.shape(), player_size);
    }

    #[test]
    fn test_players_vectorization() {
        let players = dbg!(vec![
            Player {
                direction: DiveDirection::Up,
                position: Position::Diving(5),
                held_treasures: vec![Treasure::One, Treasure::Four]
            },
            Player {
                direction: DiveDirection::Down,
                position: Position::ReturnedToSubmarine,
                held_treasures: vec![Treasure::Three],
            }
        ]);
        let players_size = &[players.unpacked_size()];
        let players_f32_ndarray = dbg!(players.into_ndarray::<f32>());
        assert_eq!(players_f32_ndarray.shape(), players_size);
        // Player1: Up.
        assert_eq!(players_f32_ndarray[[0]], 1.0);
        // Depth, and has_started / finished flags.
        assert_eq!(players_f32_ndarray[[1]], 5.0);
        assert_eq!(players_f32_ndarray[[2]], 0.0);
        assert_eq!(players_f32_ndarray[[3]], 0.0);
        // Held treasures.
        assert_eq!(players_f32_ndarray[[4]], 1.0);
        assert_eq!(players_f32_ndarray[[5]], 4.0);
        for i in 0..8 {
            assert_eq!(players_f32_ndarray[[6 + i]], 0.0);
        }
        // Player2: Down (technically impossible since returned).
        let player_two_idx = players_size[0] / 2;
        assert_eq!(players_f32_ndarray[[player_two_idx]], 0.0);
        // Returned.
        assert_eq!(players_f32_ndarray[[player_two_idx + 1]], 0.0);
        assert_eq!(players_f32_ndarray[[player_two_idx + 2]], 0.0);
        assert_eq!(players_f32_ndarray[[player_two_idx + 3]], 1.0);
        // Treasures.
        assert_eq!(players_f32_ndarray[[player_two_idx + 4]], 3.0);
        for i in 0..9 {
            assert_eq!(players_f32_ndarray[[player_two_idx + 5 + i]], 0.0);
        }
    }

    #[test]
    fn test_action_vectorization() {
        let deep_sea_action = DeepSeaAction::from(TreasureDecision::Return(Treasure::Two));
        let deep_sea_action_size = &[deep_sea_action.unpacked_size()];
        let deep_sea_action_vec = deep_sea_action.into_ndarray::<f64>();
        assert_eq!(deep_sea_action_vec.shape(), deep_sea_action_size);
        // 0 -> Take, 1 -> Ignore, 2 -> Return(One), 3 -> Return(Two).
        for i in 0..DEEP_SEA_ACTION_COUNT {
            match i {
                3 => assert_eq!(deep_sea_action_vec[[i]], 1.0),
                _ => assert_eq!(deep_sea_action_vec[[i]], 0.0),
            }
        }
    }

    #[test]
    fn test_state_action_vectorization() {
        let deep_sea_state = DeepSeaState::default();
        let state_action = DeepSeaStateActionPair {
            state: &deep_sea_state,
            action: &DeepSeaAction::DiveDirection(DiveDirection::Down),
        };
        let state_action_size = &[state_action.unpacked_size()];
        let state_action_f32_ndarray: ndarray::Array1<f32> = state_action.into_ndarray();
        assert_eq!(state_action_f32_ndarray.shape(), state_action_size);
    }

    #[test]
    fn test_state_action_into_tensor() {
        let deep_sea_state = DeepSeaState::default();
        let state_action = DeepSeaStateActionPair {
            state: &deep_sea_state,
            action: &DeepSeaAction::DiveDirection(DiveDirection::Down),
        };
        let state_action_size = &[state_action.unpacked_size()];
        let state_action_f32_tensor = state_action.into_tensordata::<f32>();
        assert_eq!(state_action_f32_tensor.shape, state_action_size);
    }
}
