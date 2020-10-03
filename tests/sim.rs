use formicarium::*;

/// Relative path of the configuration file used by these tests.
const DEFAULT_CONFIG_PATH: &str = "tests/conf.json";

/// Maximum number of generations before terminating the simulation for timeout.
const MAX_GENERATIONS_COUNT: u64 = 150000;

#[test]
fn run_simulation() {
    let mut conf = game::Conf::parse(DEFAULT_CONFIG_PATH).unwrap();

    for count in (10..=150).step_by(5) {
        conf.ants.count = count;

        let context = game::Context::new(conf.clone());
        let mut state = game::State::new(&context).unwrap();

        while !state.is_simulation_over() {
            let generation = state.env.nextgen().unwrap();
            if generation > MAX_GENERATIONS_COUNT {
                break;
            }
        }

        if state.env.generation() > MAX_GENERATIONS_COUNT {
            eprintln!("Timeout with {} ants!", count);
        } else {
            println!(
                "Simulation over after {} generations with {} ants",
                state.env.generation(),
                count
            );
        }
    }
}
