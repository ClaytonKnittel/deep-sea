use deep_sea::{
    clayton::solver::{ClaytonSolver, ClaytonSolver2},
    engine::Engine,
    error::DeepSeaResult,
};

fn run() -> DeepSeaResult {
    // let result = Engine::play_game();
    let result = Engine::evaluate_solvers::<(ClaytonSolver, ClaytonSolver2)>(10_000)?;

    println!("Result: {result:?}");

    Ok(())
}

fn main() -> DeepSeaResult {
    let result = run();
    if let Err(err) = &result {
        eprintln!("{err}");
    }
    result
}
