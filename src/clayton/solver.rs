use crate::{
    deep_sea::{DeepSea, DiveDirection, Player, Position, Tile},
    solver::{DeepSeaSolver, TreasureDecision},
    treasure::Treasure,
};

#[derive(Default)]
pub struct ClaytonSolver {}

impl ClaytonSolver {
    fn treasure_value(treasure: Treasure) -> f32 {
        match treasure {
            Treasure::One => 1.5,
            Treasure::Two => 5.5,
            Treasure::Three => 9.5,
            Treasure::Four => 13.5,
        }
    }

    fn opponents(deep_sea: &DeepSea, player_idx: usize) -> impl Iterator<Item = (usize, &Player)> {
        deep_sea
            .players()
            .iter()
            .enumerate()
            .filter(move |(idx, _)| *idx != player_idx)
            .cycle()
            .skip(player_idx)
            .take(deep_sea.players().len() - 1)
    }

    fn player_score(player: &Player) -> f32 {
        player
            .held_treasures()
            .iter()
            .cloned()
            .map(Self::treasure_value)
            .sum()
    }

    fn p_win(deep_sea: &DeepSea, player_idx: usize) -> f32 {
        let highest_opponent = Self::opponents(deep_sea, player_idx)
            .map(|(_, player)| Self::player_score(player))
            .fold(
                -1.,
                |max_score, score| if max_score > score { max_score } else { score },
            );

        if highest_opponent < Self::player_score(&deep_sea.players()[player_idx]) {
            1.
        } else {
            0.
        }
    }

    /// Probability of winning given all player head back and take no more treasure.
    fn p_win_from_pos(mut deep_sea: DeepSea) -> Vec<f32> {
        if deep_sea.done() {
            let player_scores = deep_sea.players().iter().enumerate().map(|(idx, player)| {
                if player.position() == Position::ReturnedToSubmarine {
                    (idx, Self::player_score(player))
                } else {
                    (idx, 0.)
                }
            });
            let max_value = player_scores
                .clone()
                .fold(-1., |h, (_, s)| if h > s { h } else { s });
            let num_winners = player_scores
                .clone()
                .filter(|(_, s)| *s == max_value)
                .count();

            let mut v = vec![0.; deep_sea.players().len()];
            for (idx, score) in player_scores {
                if score == max_value {
                    v[idx] = 1. / num_winners as f32;
                }
            }
            return v;
        }

        let position = deep_sea.players()[deep_sea.player_idx()].position();
        if position != Position::ReturnedToSubmarine {
            deep_sea.take_oxygen();
            deep_sea
                .move_player(
                    if position == Position::WaitingToDive {
                        DiveDirection::Down
                    } else {
                        DiveDirection::Up
                    },
                    4,
                )
                .unwrap();
        }

        deep_sea.next_player();
        Self::p_win_from_pos(deep_sea)
    }

    fn most_likely_to_win_from_pos(deep_sea: DeepSea) -> bool {
        let player_idx = deep_sea.player_idx();
        let p_win = Self::p_win_from_pos(deep_sea);
        let max_value = p_win.iter().fold(-1., |m, &s| if m > s { m } else { s });
        p_win[player_idx] == max_value
    }
}

impl DeepSeaSolver for ClaytonSolver {
    fn choose_direction(&mut self, deep_sea: &DeepSea, player_idx: usize) -> DiveDirection {
        let player = &deep_sea.players()[player_idx];
        if player.position() == Position::WaitingToDive {
            return DiveDirection::Down;
        }

        if !player.held_treasures().is_empty() {
            DiveDirection::Up
        } else {
            DiveDirection::Down
        }
    }

    fn take_treasure(&mut self, deep_sea: &DeepSea, player_idx: usize) -> TreasureDecision {
        let player = &deep_sea.players()[player_idx];
        let tile = deep_sea.path()[player.position().as_diving().unwrap()];

        if player.held_treasures().is_empty()
            && matches!(
                tile,
                Tile::Treasure(Treasure::Two | Treasure::Three | Treasure::Four)
            )
        {
            TreasureDecision::Take
        } else {
            TreasureDecision::Ignore
        }
    }
}

#[derive(Default)]
pub struct ClaytonSolver2;

impl DeepSeaSolver for ClaytonSolver2 {
    fn choose_direction(&mut self, deep_sea: &DeepSea, player_idx: usize) -> DiveDirection {
        let player = &deep_sea.players()[player_idx];
        if player.position() == Position::WaitingToDive {
            return DiveDirection::Down;
        }

        if !player.held_treasures().is_empty() {
            DiveDirection::Up
        } else {
            DiveDirection::Down
        }
    }

    fn take_treasure(&mut self, deep_sea: &DeepSea, player_idx: usize) -> TreasureDecision {
        let player = &deep_sea.players()[player_idx];
        let tile = deep_sea.path()[player.position().as_diving().unwrap()];

        if (player.held_treasures().is_empty()
            && matches!(
                tile,
                Tile::Treasure(Treasure::Two | Treasure::Three | Treasure::Four)
            ))
            || (!ClaytonSolver::most_likely_to_win_from_pos(deep_sea.clone())
                && tile != Tile::Empty)
        {
            TreasureDecision::Take
        } else {
            TreasureDecision::Ignore
        }
    }
}
