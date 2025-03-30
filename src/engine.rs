use rand::Rng;

use crate::{
    deep_sea::{DeepSea, DiveDirection, Position, Tile},
    error::DeepSeaResult,
    random_solver::RandomSolver,
    solver::DeepSeaSolver,
    treasure::{Treasure, TreasureValueAssigner},
};

pub struct Engine {
    state: DeepSea,
    players: Vec<Box<dyn DeepSeaSolver>>,
}

impl Engine {
    pub fn new(path: Vec<Tile>, players: Vec<Box<dyn DeepSeaSolver>>) -> Self {
        Self {
            state: DeepSea::new(path, players.len()),
            players,
        }
    }

    pub fn make_default_game(players: Vec<Box<dyn DeepSeaSolver>>) -> Self {
        Self::new(
            (0..8)
                .map(|_| Tile::Treasure(Treasure::One))
                .chain((0..8).map(|_| Tile::Treasure(Treasure::Two)))
                .chain((0..8).map(|_| Tile::Treasure(Treasure::Three)))
                .chain((0..8).map(|_| Tile::Treasure(Treasure::Four)))
                .collect(),
            players,
        )
    }

    fn roll_dice() -> u32 {
        let mut rng = rand::rng();
        let d1 = rng.random_range(1..=3);
        let d2 = rng.random_range(1..=3);
        d1 + d2
    }

    fn take_turn(&mut self) -> DeepSeaResult {
        let player_idx = self.state.player_idx();
        let player = &self.state.players()[player_idx];
        if player.position() == Position::ReturnedToSubmarine {
            self.state.next_player();
            return Ok(());
        }

        self.state.take_oxygen();

        let player_agent = &mut self.players[player_idx];

        let player = &self.state.players()[player_idx];
        let direction = if player.direction() == DiveDirection::Down {
            player_agent.choose_direction(&self.state, player_idx)
        } else {
            DiveDirection::Up
        };

        let dice_roll = Self::roll_dice();
        self.state.move_player(direction, dice_roll)?;

        let player = &self.state.players()[player_idx];
        if let Position::Diving(_) = player.position() {
            self.state
                .take_treasure(player_agent.take_treasure(&self.state, player_idx))?;
        }

        self.state.next_player();
        Ok(())
    }

    fn score_game(&self) -> Vec<u32> {
        debug_assert!(self.state.done());
        let mut value_assigner = TreasureValueAssigner::new();
        self.state
            .players()
            .iter()
            .map(|player| {
                if player.position() == Position::ReturnedToSubmarine {
                    player
                        .held_treasures()
                        .iter()
                        .map(|&treasure| value_assigner.assign_value(treasure))
                        .sum()
                } else {
                    0
                }
            })
            .collect()
    }

    pub fn play_game() -> DeepSeaResult {
        let mut s = Self::make_default_game(vec![
            Box::new(RandomSolver),
            Box::new(RandomSolver),
            Box::new(RandomSolver),
            Box::new(RandomSolver),
            Box::new(RandomSolver),
            Box::new(RandomSolver),
        ]);

        println!("{}", s.state);
        while !s.state.done() {
            s.take_turn()?;
            println!("{}", s.state);
        }

        for (idx, score) in s.score_game().into_iter().enumerate() {
            println!("{idx}: {score}");
        }

        Ok(())
    }
}
