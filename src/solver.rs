use itertools::Itertools;
use rand::seq::SliceRandom;
use strum_macros::EnumCount;

use crate::{
    deep_sea::{DeepSea, DiveDirection},
    treasure::Treasure,
};

#[derive(Clone, Copy, Debug, EnumCount, PartialEq, Eq, Hash)]
pub enum TreasureDecision {
    Ignore,
    Take,
    Return(Treasure),
}

pub trait DeepSeaSolver {
    fn choose_direction(&mut self, deep_sea: &DeepSea, player_idx: usize) -> DiveDirection;

    fn take_treasure(&mut self, deep_sea: &DeepSea, player_idx: usize) -> TreasureDecision;
}

pub trait IntoSolvers {
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>>;

    fn initialize_shuffled_solvers() -> (Vec<Box<dyn DeepSeaSolver>>, Vec<usize>) {
        let mut solvers = Self::initialize_solvers()
            .into_iter()
            .enumerate()
            .collect_vec();
        solvers.shuffle(&mut rand::rng());

        solvers.into_iter().fold(
            (vec![], vec![]),
            |(mut solvers, mut idxs), (idx, solver)| {
                solvers.push(solver);
                idxs.push(idx);
                (solvers, idxs)
            },
        )
    }

    fn num_solvers() -> usize;
}

impl<T> IntoSolvers for T
where
    T: DeepSeaSolver + Default + 'static,
{
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>> {
        vec![Box::new(T::default())]
    }

    fn num_solvers() -> usize {
        1
    }
}

impl<T, U> IntoSolvers for (T, U)
where
    T: DeepSeaSolver + Default + 'static,
    U: DeepSeaSolver + Default + 'static,
{
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>> {
        vec![Box::new(T::default()), Box::new(U::default())]
    }

    fn num_solvers() -> usize {
        2
    }
}

impl<T, U, V> IntoSolvers for (T, U, V)
where
    T: DeepSeaSolver + Default + 'static,
    U: DeepSeaSolver + Default + 'static,
    V: DeepSeaSolver + Default + 'static,
{
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>> {
        vec![
            Box::new(T::default()),
            Box::new(U::default()),
            Box::new(V::default()),
        ]
    }

    fn num_solvers() -> usize {
        3
    }
}

impl<T, U, V, W> IntoSolvers for (T, U, V, W)
where
    T: DeepSeaSolver + Default + 'static,
    U: DeepSeaSolver + Default + 'static,
    V: DeepSeaSolver + Default + 'static,
    W: DeepSeaSolver + Default + 'static,
{
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>> {
        vec![
            Box::new(T::default()),
            Box::new(U::default()),
            Box::new(V::default()),
            Box::new(W::default()),
        ]
    }

    fn num_solvers() -> usize {
        4
    }
}

impl<T, U, V, W, X> IntoSolvers for (T, U, V, W, X)
where
    T: DeepSeaSolver + Default + 'static,
    U: DeepSeaSolver + Default + 'static,
    V: DeepSeaSolver + Default + 'static,
    W: DeepSeaSolver + Default + 'static,
    X: DeepSeaSolver + Default + 'static,
{
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>> {
        vec![
            Box::new(T::default()),
            Box::new(U::default()),
            Box::new(V::default()),
            Box::new(W::default()),
            Box::new(X::default()),
        ]
    }

    fn num_solvers() -> usize {
        5
    }
}

impl<T, U, V, W, X, Y> IntoSolvers for (T, U, V, W, X, Y)
where
    T: DeepSeaSolver + Default + 'static,
    U: DeepSeaSolver + Default + 'static,
    V: DeepSeaSolver + Default + 'static,
    W: DeepSeaSolver + Default + 'static,
    X: DeepSeaSolver + Default + 'static,
    Y: DeepSeaSolver + Default + 'static,
{
    fn initialize_solvers() -> Vec<Box<dyn DeepSeaSolver>> {
        vec![
            Box::new(T::default()),
            Box::new(U::default()),
            Box::new(V::default()),
            Box::new(W::default()),
            Box::new(X::default()),
            Box::new(Y::default()),
        ]
    }

    fn num_solvers() -> usize {
        6
    }
}
