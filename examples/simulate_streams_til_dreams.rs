use structopt::StructOpt;

use mc_sim::sim::{Simulation, SimulationGoalsBuilder};

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long, default_value = "32")]
    threads: u32,

    #[structopt(
        short,
        long,
        default_value = "0.000000000000000000005902209912719003371976488112274"
    )]
    p_value: f64,
}

fn main() {
    let args = Cli::from_args();
    let goals = SimulationGoalsBuilder::new().add_runs(22, 10, 7).goals();

    let simulation = Simulation::new(goals.clone(), args.threads);
    simulation.run_to_p_value(args.p_value);
}
