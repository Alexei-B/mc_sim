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

    #[structopt(long, default_value = "./data/barters.csv")]
    output_path: String,
}

fn main() {
    let args = Cli::from_args();
    let goals = SimulationGoalsBuilder::new().add_runs(17, 10, 0).goals();

    let simulation = Simulation::new(goals.clone(), args.threads);
    let data = simulation.simulate_n_times(args.cycles);
    let records = count_ender_pearl_simulation_data(&goals, &data);
    write_simulation_data(&records, args.output_path);
}

fn count_ender_pearl_simulation_data(
    goals: &SimulationGoals,
    data: &[StreamResults],
) -> Vec<BarterRecord> {
    let ender_pearl_target_total = goals
        .streams
        .iter()
        .map(|s| s.iter().map(|r| r.target_pearls).sum::<u32>())
        .sum();

    let ender_pearl_target_per_run =
        ender_pearl_target_total / goals.streams.iter().map(|s| s.len() as u32).sum::<u32>();

    let barter_drop_list =
        drop_list::barter_drop_list(ender_pearl_target_total, ender_pearl_target_per_run);
    let mut table = HashMap::<u32, SimulationRecordData>::new();

    for result in data {
        match table.get_mut(&result.total_barters) {
            None => {
                table.insert(
                    result.total_barters,
                    SimulationRecordData::new(result.pearl_probability(&barter_drop_list)),
                );
            }
            Some(record) => record.count += 1,
        }
    }

    let mut records: Vec<BarterRecord> = table
        .into_iter()
        .map(|(k, v)| BarterRecord {
            barters: k,
            count: v.count,
            frequency: v.count as f64 / data.len() as f64,
            estimated_probability: v.estimated_probability,
        })
        .collect();

    records.sort_by(|lhs, rhs| lhs.barters.cmp(&rhs.barters));
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
struct BarterRecord {
    pub barters: u32,
    pub estimated_probability: f64,
    pub count: u64,
    pub frequency: f64,
}
