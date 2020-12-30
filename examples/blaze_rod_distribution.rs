#[macro_use]
extern crate serde_derive;

use serde::Serialize;
use std::collections::HashMap;
use structopt::StructOpt;

use mc_sim::drop_list;
use mc_sim::sim::{Simulation, SimulationGoals, SimulationGoalsBuilder};
use mc_sim::stream::StreamResults;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long, default_value = "32")]
    threads: u32,

    #[structopt(short, long, default_value = "1000000")]
    cycles: u64,

    #[structopt(long, default_value = "./data/blazes.csv")]
    output_path: String,
}

fn main() {
    let args = Cli::from_args();
    let goals = SimulationGoalsBuilder::new()
        .add_run(0, 6)
        .add_run(0, 7)
        .add_run(0, 8)
        .add_run(0, 7)
        .add_run(0, 8)
        .add_run(0, 8)
        .add_run(0, 5)
        .add_run(0, 3)
        .add_run(0, 1)
        .add_run(0, 8)
        .add_run(0, 8)
        .add_run(0, 6)
        .add_run(0, 8)
        .add_run(0, 6)
        .add_run(0, 3)
        .add_run(0, 1)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 3)
        .add_run(0, 8)
        .add_run(0, 8)
        .add_run(0, 6)
        .add_run(0, 8)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 7)
        .add_run(0, 8)
        .goals();

    let simulation = Simulation::new(goals.clone(), args.threads);
    let data = simulation.simulate_n_times(args.cycles);
    let records = count_blaze_rod_simulation_data(&goals, &data);
    write_simulation_data(&records, args.output_path);
}

fn count_blaze_rod_simulation_data(
    goals: &SimulationGoals,
    data: &[StreamResults],
) -> Vec<FightRecord> {
    let blaze_rod_target = goals
        .streams
        .iter()
        .map(|s| s.iter().map(|r| r.target_rods).sum::<u32>())
        .sum();

    let blaze_drop_list = drop_list::blaze_drop_list(blaze_rod_target);
    let mut table = HashMap::<u32, SimulationRecordData>::new();

    for result in data {
        match table.get_mut(&result.total_fights) {
            None => {
                table.insert(
                    result.total_fights,
                    SimulationRecordData::new(result.rod_probability(&blaze_drop_list)),
                );
            }
            Some(record) => record.count += 1,
        }
    }

    let mut records: Vec<FightRecord> = table
        .into_iter()
        .map(|(k, v)| FightRecord {
            blazes: k,
            count: v.count,
            frequency: v.count as f64 / data.len() as f64,
            estimated_probability: v.estimated_probability,
        })
        .collect();

    records.sort_by(|lhs, rhs| lhs.blazes.cmp(&rhs.blazes));
    records
}

fn write_simulation_data<T>(data: &[T], path: String)
where
    T: Serialize,
{
    let mut writer = csv::Writer::from_path(&path).unwrap();

    for record in data {
        writer.serialize(record).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SimulationRecordData {
    pub estimated_probability: f64,
    pub count: u64,
}

impl SimulationRecordData {
    pub fn default() -> Self {
        Self {
            estimated_probability: 0.0,
            count: 0,
        }
    }

    pub fn new(estimated_probability: f64) -> Self {
        Self {
            estimated_probability,
            count: 1,
        }
    }
}

impl std::default::Default for SimulationRecordData {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FightRecord {
    pub blazes: u32,
    pub estimated_probability: f64,
    pub count: u64,
    pub frequency: f64,
}
