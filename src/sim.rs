use crate::drop::DropSim;
use crate::drop_list::{self, DropList};
use crate::run::RunGoals;
use crate::stats::{BlazeRodDistribution, EnderPearlDistribution};
use crate::stream::{Stream, StreamResults};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;
use std::{thread::JoinHandle, time::Instant};

/// The goals of a simulation of speed run streams.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulationGoals {
    pub streams: Vec<Vec<RunGoals>>,
}

impl SimulationGoals {
    /// Create simulation goals from a list of streams.
    pub fn new(streams: Vec<Vec<RunGoals>>) -> Self {
        Self { streams }
    }

    /// Create simulation goals for a number of streams that repeat a set of runs a specific number of times.
    pub fn new_repeat_streams(streams: u64, run_goals: Vec<RunGoals>) -> Self {
        Self {
            streams: (0..streams).map(|_| run_goals.clone()).collect(),
        }
    }

    /// Consume the simulation goals and get out all of the streams run goal lists.
    pub fn into_streams(self) -> Vec<Vec<RunGoals>> {
        self.streams
    }
}

/// Builds simulation goals from chain calls, to make simulation goals easier to configure.
pub struct SimulationGoalsBuilder {
    streams: Vec<Vec<RunGoals>>,
}

impl SimulationGoalsBuilder {
    /// Create a simulation goals builder.
    /// ```
    /// # use mc_sim::sim::*;
    /// let goals = SimulationGoalsBuilder::new()
    ///     .add_stream()
    ///     .add_run(10, 7)
    ///     .add_run(10, 6)
    ///     .add_run(10, 8)
    ///     .add_run(10, 7)
    ///     .add_stream()
    ///     .add_runs(3, 10, 7)
    ///     .goals();
    ///
    /// assert_eq!(goals.streams.len(), 2);
    /// assert_eq!(goals.streams[0].len(), 4);
    /// assert_eq!(goals.streams[1].len(), 3);
    /// assert_eq!(goals.streams[0][1].target_pearls, 10);
    /// assert_eq!(goals.streams[0][1].target_rods, 6);
    /// ```
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
        }
    }

    /// Get the simulation goals, consumes the builder.
    pub fn goals(self) -> SimulationGoals {
        SimulationGoals::new(self.streams)
    }

    /// Add a stream to the simulation.
    pub fn add_stream(mut self) -> Self {
        self.streams.push(Vec::new());
        self
    }

    /// Add a run to the simulation.
    pub fn add_run(mut self, target_pearls: u32, target_rods: u32) -> Self {
        if self.streams.len() == 0 {
            return self.add_stream().add_run(target_pearls, target_rods);
        }

        self.streams.last_mut().unwrap().push(RunGoals {
            target_pearls,
            target_rods,
        });
        self
    }

    /// Add a set of runs to the simulation.
    pub fn add_runs(mut self, runs: u32, target_pearls: u32, target_rods: u32) -> Self {
        if self.streams.len() == 0 {
            return self.add_stream().add_runs(runs, target_pearls, target_rods);
        }

        for _ in 0..runs {
            self.streams.last_mut().unwrap().push(RunGoals {
                target_pearls,
                target_rods,
            });
        }
        self
    }
}

/// A single thread used in simulating minecraft runs.
/// All the actual work is done on worker threads, not on the main thread.
struct SimulationThread {
    luckiest_stream: Arc<RwLock<Option<Stream>>>,
    simulations: Arc<RwLock<u64>>,
    thread: JoinHandle<Vec<StreamResults>>,
}

impl SimulationThread {
    /// Create a simulation thread.
    /// The `completed` locked-bool is used to stop the thread.
    pub fn new(
        name: String,
        completed: Arc<RwLock<bool>>,
        goals: SimulationGoals,
        barter_drop_list: DropList<EnderPearlDistribution>,
        blaze_drop_list: DropList<BlazeRodDistribution>,
    ) -> Self {
        let luckiest_stream = Arc::new(RwLock::new(None));
        let simulations = Arc::new(RwLock::new(0));

        Self {
            luckiest_stream: Arc::clone(&luckiest_stream),
            simulations: Arc::clone(&simulations),
            thread: thread::Builder::new()
                .name(name)
                .spawn(move || {
                    SimulationThread::run(
                        goals,
                        completed,
                        luckiest_stream,
                        simulations,
                        barter_drop_list,
                        blaze_drop_list,
                    )
                })
                .unwrap(),
        }
    }

    /// The number of simulations that have been completed.
    /// This is only updated every now and then while running, so it is approximate
    /// until the thread has been joined.
    pub fn simulations(&self) -> u64 {
        *self.simulations.read().unwrap()
    }

    /// The luckiest stream seen so far by this worker thread.
    pub fn luckiest_stream(&self) -> RwLockReadGuard<Option<Stream>> {
        self.luckiest_stream.read().unwrap()
    }

    /// Consumes the simulation thread into a join handle, which provides the stream results.
    pub fn into_thread(self) -> JoinHandle<Vec<StreamResults>> {
        self.thread
    }

    /// Runs the simulation.
    fn run(
        goals: SimulationGoals,
        completed: Arc<RwLock<bool>>,
        luckiest_stream: Arc<RwLock<Option<Stream>>>,
        simulations: Arc<RwLock<u64>>,
        barter_drop_list: DropList<EnderPearlDistribution>,
        blaze_drop_list: DropList<BlazeRodDistribution>,
    ) -> Vec<StreamResults> {
        // Each thread uses it's own drop simulators so that they keep the RNG on that thread.
        let mut barter_drop_sim = DropSim::new(barter_drop_list.list_clone());
        let mut blaze_drop_sim = DropSim::new(blaze_drop_list.list_clone());

        // The results of running a simulation are just simple StreamResults.
        // The entire streams could be stored and returned, but that would eat memory fast.
        let mut data = Vec::<StreamResults>::new();
        let mut tries = 0;
        let mut last_update = Instant::now();

        // Tracks the best stream so far. Starts as unreasonably bad luck, so that we immediately replace this.
        let mut personal_best_luck = 1.0;
        let mut personal_best_barters = 999999;
        let mut personal_best_fights = 999999;

        loop {
            // Simulate our list of streams.
            let streams: Vec<Stream> = goals
                .clone()
                .into_streams()
                .into_iter()
                .map(|run_goals| {
                    Stream::simulate(&mut barter_drop_sim, &mut blaze_drop_sim, run_goals)
                })
                .collect();

            // Add the data to our results.
            for stream in streams {
                let results = stream.results();
                data.push(results.clone());
                tries += 1;

                // Does it look like we might have beaten our PB?
                if personal_best_barters > results.total_barters
                    || personal_best_fights > results.total_fights
                {
                    let luck = results.luck(&barter_drop_list, &blaze_drop_list);

                    // Only actually grab the luckiest stream rwlock when we know we've beaten our PB.
                    if personal_best_luck > luck {
                        personal_best_luck = luck;
                        personal_best_barters = results.total_barters;
                        personal_best_fights = results.total_fights;

                        *luckiest_stream.write().unwrap() = Some(stream.clone());
                    }
                }
            }

            // Every now and then, update the number of simulations run
            // and check if we should stop because the completed flag is set.
            // This is done to avoid hogging the rwlocks.
            if last_update.elapsed().as_millis() >= 2000 {
                last_update = Instant::now();
                *simulations.write().unwrap() = tries;

                if *completed.read().unwrap() {
                    break;
                }
            }
        }

        data
    }
}

/// A simulation of a series of streams of speed runs, distributed over worker threads.
pub struct Simulation {
    goals: SimulationGoals,
    completed: Arc<RwLock<bool>>,
    workers: Vec<SimulationThread>,
    barter_drop_list: DropList<EnderPearlDistribution>,
    blaze_drop_list: DropList<BlazeRodDistribution>,
}

impl Simulation {
    /// Create a simulation.
    /// ```
    /// # use mc_sim::sim::*;
    /// let goals = SimulationGoalsBuilder::new().add_runs(5, 10, 7).goals();
    /// let simulation = Simulation::new(goals, 4);
    /// let results = simulation.simulate_n_times(100);
    /// # assert!(results.len() >= 100);
    /// ```
    pub fn new(goals: SimulationGoals, thread_count: u32) -> Self {
        let completed = Arc::new(RwLock::new(false));
        let (barter_drop_list, blaze_drop_list) = Simulation::drop_lists(&goals);

        Self {
            barter_drop_list: barter_drop_list.clone(),
            blaze_drop_list: blaze_drop_list.clone(),
            goals: goals.clone(),
            completed: Arc::clone(&completed),
            workers: (0..thread_count)
                .map(|id| {
                    SimulationThread::new(
                        format!("Simulation Worker Thread #{}", id),
                        Arc::clone(&completed),
                        goals.clone(),
                        barter_drop_list.clone(),
                        blaze_drop_list.clone(),
                    )
                })
                .collect(),
        }
    }

    /// Run the simulation for a given number of cycles and get the results.
    /// This will consume the simulator.
    pub fn simulate_n_times(self, cycles: u64) -> Vec<StreamResults> {
        let mut last_printed = Instant::now();
        let start = Instant::now();

        loop {
            if last_printed.elapsed().as_millis() >= 5000 {
                last_printed = Instant::now();
                self.print_update_with_progress(&start, cycles * self.goals.streams.len() as u64);

                if self.simulations() >= cycles {
                    *self.completed.write().unwrap() = true;
                    break;
                }
            }

            thread::yield_now();
        }

        self.into_results()
    }

    /// Run the simulation until a desired p-value is reached.
    /// I.E. The luckiest run seen, is as lucky, or luckier than the given p-value.
    pub fn run_to_p_value(self, p_value: f64) -> StreamResults {
        let mut last_printed = Instant::now();
        let start = Instant::now();

        loop {
            if last_printed.elapsed().as_millis() >= 5000 {
                last_printed = Instant::now();
                self.print_update_with_target(&start, p_value);

                if let Some(results) = self.luckiest_stream() {
                    if results.luck(&self.barter_drop_list, &self.blaze_drop_list) <= p_value {
                        *self.completed.write().unwrap() = true;
                        break;
                    }
                }
            }

            thread::yield_now();
        }

        self.luckiest_stream().unwrap()
    }

    /// The goals of the simulation.
    pub fn goals(&self) -> &SimulationGoals {
        &self.goals
    }

    /// Prints a message updating the user on the status of the simulation.
    fn print_update_with_progress(&self, start: &Instant, target_num_streams: u64) {
        let luckiest_stream = self.luckiest_stream();
        let streams = self.simulations() * self.goals.streams.len() as u64;
        let streams_per_second = streams / start.elapsed().as_secs();
        let completed = streams as f32 / target_num_streams as f32;

        let time_remaining: humantime::Duration = std::time::Duration::from_secs(
            (target_num_streams - std::cmp::min(streams, target_num_streams)) as u64
                / std::cmp::max(1, streams_per_second),
        )
        .into();

        let total_time_estimate: humantime::Duration = std::time::Duration::from_secs(
            target_num_streams / std::cmp::max(1, streams_per_second),
        )
        .into();

        if let Some(luckiest_stream) = luckiest_stream {
            println!(
                "luckiest stream: {} ({} barters, {} fights), streams simulated: {}/{}, streams per second: {}, complete: {}%, est: {}/{}",
                luckiest_stream.luck(&self.barter_drop_list, &self.blaze_drop_list),
                luckiest_stream.total_barters,
                luckiest_stream.total_fights,
                streams,
                target_num_streams,
                streams_per_second,
                completed * 100.0,
                time_remaining,
                total_time_estimate,
            );
        } else {
            println!(
                "streams simulated: {}/{}, streams per second: {}, complete: {}%, est: {}/{}",
                streams,
                target_num_streams,
                streams_per_second,
                completed * 100.0,
                time_remaining,
                total_time_estimate,
            );
        }
    }

    /// Prints a message updating the user on the status of the simulation.
    fn print_update_with_target(&self, start: &Instant, target_p_value: f64) {
        let luckiest_stream = self.luckiest_stream();
        let streams = self.simulations() * self.goals.streams.len() as u64;
        let streams_per_second = streams / start.elapsed().as_secs();
        let time_elapsed: humantime::Duration = start.elapsed().into();

        if let Some(luckiest_stream) = luckiest_stream {
            println!(
                "luckiest stream: {} ({} barters, {} fights), target luck: {}, streams simulated: {}, streams per second: {}, elapsed: {}",
                luckiest_stream.luck(&self.barter_drop_list, &self.blaze_drop_list),
                luckiest_stream.total_barters,
                luckiest_stream.total_fights,
                target_p_value,
                streams,
                streams_per_second,
                time_elapsed,
            );
        } else {
            println!(
                "target luck: {}, streams simulated: {}, streams per second: {}, elapsed: {}",
                target_p_value,
                streams,
                streams_per_second,
                time_elapsed,
            );
        }
    }

    /// Get the number of simulations that have been run in total from all worker threads (approximated while they are running).
    fn simulations(&self) -> u64 {
        self.workers.iter().map(|worker| worker.simulations()).sum()
    }

    /// Get the luckiest stream that has been simulated from across all worker threads (approximated while they are running).
    fn luckiest_stream(&self) -> Option<StreamResults> {
        self.workers
            .iter()
            .map(|worker| {
                worker
                    .luckiest_stream()
                    .as_ref()
                    .map(|stream| stream.results())
            })
            .filter(|results| results.is_some())
            .map(|results| results.unwrap())
            .min_by(|lhs, rhs| {
                lhs.luck(&self.barter_drop_list, &self.blaze_drop_list)
                    .partial_cmp(&rhs.luck(&self.barter_drop_list, &self.blaze_drop_list))
                    .unwrap()
            })
    }

    /// Consumes the simulation and produces stream results.
    fn into_results(self) -> Vec<StreamResults> {
        self.workers
            .into_iter()
            .flat_map(|worker| worker.into_thread().join().unwrap())
            .collect()
    }

    fn drop_lists(
        goals: &SimulationGoals,
    ) -> (
        DropList<EnderPearlDistribution>,
        DropList<BlazeRodDistribution>,
    ) {
        let ender_pearl_target_total = goals
            .streams
            .iter()
            .map(|s| s.iter().map(|r| r.target_pearls).sum::<u32>())
            .sum();

        let ender_pearl_target_per_run =
            ender_pearl_target_total / goals.streams.iter().map(|s| s.len() as u32).sum::<u32>();

        let barter_drop_list =
            drop_list::barter_drop_list(ender_pearl_target_total, ender_pearl_target_per_run);

        let blaze_rod_target = goals
            .streams
            .iter()
            .map(|s| s.iter().map(|r| r.target_rods).sum::<u32>())
            .sum();

        let blaze_drop_list = drop_list::blaze_drop_list(blaze_rod_target);

        (barter_drop_list, blaze_drop_list)
    }
}
