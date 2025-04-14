use criterion::{Criterion, black_box, criterion_group, criterion_main};

use deep_sea::{
    deep_sea::DiveDirection,
    deep_sea_vectorization::{DeepSeaAction, DeepSeaState, DeepSeaStateActionPair},
    ml::vectorization::*,
};

fn bench_state_action_into_tensordata(c: &mut Criterion) {
    let default_state = black_box(DeepSeaState::default());
    c.bench_function("starting state into tensordata", |b| {
        b.iter_batched(
            || {
                black_box(DeepSeaStateActionPair {
                    state: &default_state,
                    action: &DeepSeaAction::DiveDirection(DiveDirection::Down),
                })
            },
            |data| data.into_tensordata::<f32>(),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_state_action_vectorization(c: &mut Criterion) {
    let default_state = black_box(DeepSeaState::default());
    c.bench_function("starting state vectorization", |b| {
        b.iter_batched(
            || {
                black_box(DeepSeaStateActionPair {
                    state: &default_state,
                    action: &DeepSeaAction::DiveDirection(DiveDirection::Down),
                })
            },
            |data| data.into_ndarray::<f32>(),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_state_action_unpacking(c: &mut Criterion) {
    let default_state = black_box(DeepSeaState::default());
    let state_action = black_box(DeepSeaStateActionPair {
        state: &default_state,
        action: &DeepSeaAction::DiveDirection(DiveDirection::Down),
    });
    c.bench_function("starting state unpacking", |b| {
        b.iter(|| state_action.unpack::<f32>())
    });
}

criterion_group!(
    benches,
    bench_state_action_into_tensordata,
    bench_state_action_unpacking,
    bench_state_action_vectorization
);
criterion_main!(benches);
