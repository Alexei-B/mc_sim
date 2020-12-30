use crate::drop::DropSim;
use crate::drop_list::DropList;
use crate::run::{Run, RunGoals, RunSim};
use crate::stats::{BlazeRodDistribution, EnderPearlDistribution};

/// A summary of the results of a stream, targeted around answering questions about
/// how lucky we got with piglins barters and blaze fights specifically.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamResults {
    pub number_of_runs: u32,
    pub total_barters: u32,
    pub total_fights: u32,
    pub successful_barters: u32,
    pub successful_fights: u32,
    total_target_pearls: u32,
    average_target_pearls_per_run: u32,
    total_target_rods: u32,
}

impl StreamResults {
    /// Creates stream results from the goals of all of the runs in the stream,
    /// and the total number of barters and fights that stream had to get to those goals.
    pub fn new(
        goals: &[RunGoals],
        total_barters: u32,
        total_fights: u32,
        successful_barters: u32,
        successful_fights: u32,
    ) -> Self {
        let total_target_pearls = goals.iter().map(|r| r.target_pearls).sum();
        let total_target_rods = goals.iter().map(|r| r.target_rods).sum();
        let average_target_pearls_per_run = total_target_pearls / goals.len() as u32;

        Self {
            total_barters,
            total_fights,
            successful_barters,
            successful_fights,
            number_of_runs: goals.len() as u32,
            total_target_pearls,
            average_target_pearls_per_run,
            total_target_rods,
        }
    }

    /// Estimates a p-value for the stream results being this lucky.
    /// Lucky meaning fewest barters and blaze fights, accounting for how
    /// likely each of those are.
    ///
    /// A lower p-value means that this stream is more lucky.
    /// ```
    /// # use mc_sim::drop_list;
    /// # use mc_sim::sim::*;
    /// # use mc_sim::stream::*;
    /// let (runs, pearls, rods) = (22, 10, 7);
    /// let goals = SimulationGoalsBuilder::new().add_runs(runs, pearls, rods).goals();
    /// let (target_pearls, target_rods) = (runs * pearls, runs * rods);
    /// let (total_barters, total_fights) = (937, 308);
    /// let (successful_barters, successful_fights) = ((target_pearls * 20 * 25) / (53 * 423), target_rods);
    /// let results = StreamResults::new(&goals.streams[0], total_barters, total_fights, successful_barters, successful_fights);
    /// assert_eq!(results.pearl_luck(&drop_list::barter_drop_list(target_pearls, pearls)), 0.5016436716111609);
    /// assert_eq!(results.rod_luck(&drop_list::blaze_drop_list(target_rods)), 0.5227134024692426);
    /// assert_eq!(results.luck(&drop_list::barter_drop_list(target_pearls, pearls), &drop_list::blaze_drop_list(target_rods)), 0.2622158704150333);
    /// ```
    pub fn luck(
        &self,
        barter_drop_list: &DropList<EnderPearlDistribution>,
        blaze_drop_list: &DropList<BlazeRodDistribution>,
    ) -> f64 {
        self.pearl_luck(barter_drop_list) * self.rod_luck(blaze_drop_list)
    }

    /// Estimates a p-value for the stream results exact number of barters and fights.
    /// Probability meaning how likely this outcome was, not how lucky it was. See: [luck](StreamResults::luck)
    /// ```
    /// # use mc_sim::drop_list;
    /// # use mc_sim::sim::*;
    /// # use mc_sim::stream::*;
    /// let (runs, pearls, rods) = (22, 10, 7);
    /// let goals = SimulationGoalsBuilder::new().add_runs(runs, pearls, rods).goals();
    /// let (target_pearls, target_rods) = (runs * pearls, runs * rods);
    /// let (total_barters, total_fights) = (937, 308);
    /// let (successful_barters, successful_fights) = ((target_pearls * 20 * 25) / (53 * 423), target_rods);
    /// let results = StreamResults::new(&goals.streams[0], total_barters, total_fights, successful_barters, successful_fights);
    /// assert_eq!(results.pearl_probability(&drop_list::barter_drop_list(target_pearls, pearls)), 0.0028413877468180587);
    /// assert_eq!(results.rod_probability(&drop_list::blaze_drop_list(target_rods)), 0.022713402469194337);
    /// assert_eq!(results.probability(&drop_list::barter_drop_list(target_pearls, pearls), &drop_list::blaze_drop_list(target_rods)), 0.00006453758346451583);
    /// ```
    pub fn probability(
        &self,
        barter_drop_list: &DropList<EnderPearlDistribution>,
        blaze_drop_list: &DropList<BlazeRodDistribution>,
    ) -> f64 {
        self.pearl_probability(barter_drop_list) * self.rod_probability(blaze_drop_list)
    }

    /// Estimates the stream results luck specifically for ender pearls. See: [luck](StreamResults::luck)
    pub fn pearl_luck(&self, barter_drop_list: &DropList<EnderPearlDistribution>) -> f64 {
        if self.total_target_pearls == 0 {
            return 1.0;
        }

        barter_drop_list
            .distribution()
            .unwrap()
            .luck(self.total_barters, self.successful_barters)
    }

    /// Estimates the stream results luck specifically for blaze rods. See: [luck](StreamResults::luck)
    pub fn rod_luck(&self, blaze_drop_list: &DropList<BlazeRodDistribution>) -> f64 {
        if self.total_target_rods == 0 {
            return 1.0;
        }

        blaze_drop_list
            .distribution()
            .unwrap()
            .luck(self.total_fights)
    }

    /// Estimates the stream results probability specifically for ender pearls. See: [probability](StreamResults::probability)
    pub fn pearl_probability(&self, barter_drop_list: &DropList<EnderPearlDistribution>) -> f64 {
        if self.total_target_pearls == 0 {
            return 0.0;
        }

        barter_drop_list
            .distribution()
            .unwrap()
            .probability(self.total_barters, self.successful_barters)
    }

    /// Estimates the stream results luck specifically for blaze rods. See: [probability](StreamResults::probability)
    pub fn rod_probability(&self, blaze_drop_list: &DropList<BlazeRodDistribution>) -> f64 {
        if self.total_target_rods == 0 {
            return 0.0;
        }

        blaze_drop_list
            .distribution()
            .unwrap()
            .probability(self.total_fights)
    }
}

/// A single 'stream' of minecraft speed runs.
/// I.E. A list of speed runs.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Stream {
    pub runs: Vec<Run>,
    pub goals: Vec<RunGoals>,
}

impl Stream {
    /// Simulate the stream from drop lists for bartering and blazes, and a list of goals per run.
    /// ```
    /// # use mc_sim::drop::*;
    /// # use mc_sim::drop_list;
    /// # use mc_sim::run::*;
    /// # use mc_sim::stream::*;
    /// let mut barter_drop_sim = DropSim::new(drop_list::barter_drop_list(10, 10).list_clone());
    /// let mut blaze_drop_sim = DropSim::new(drop_list::blaze_drop_list(7).list_clone());
    /// let goals = vec![
    ///     RunGoals { target_pearls: 10, target_rods: 7 },
    ///     RunGoals { target_pearls: 10, target_rods: 6 },
    ///     RunGoals { target_pearls: 10, target_rods: 8 },
    ///     RunGoals { target_pearls: 10, target_rods: 7 },
    /// ];
    ///
    /// let stream = Stream::simulate(&mut barter_drop_sim, &mut blaze_drop_sim, goals);
    ///
    /// assert!(stream.total_pearls() >= 40);
    /// assert!(stream.total_rods() >= 28);
    /// assert_eq!(stream.runs.len(), 4);
    /// assert!(stream.runs[2].total_rods() >= 8);
    /// ```
    pub fn simulate(
        barter_drop_sim: &mut DropSim,
        blaze_drop_sim: &mut DropSim,
        goals: Vec<RunGoals>,
    ) -> Self {
        let runs = (0..goals.len())
            .map(|run| Stream::simulate_run(barter_drop_sim, blaze_drop_sim, &goals, run))
            .collect();

        Self { goals, runs }
    }

    /// The total number of barters made across all runs in the stream.
    pub fn total_barters(&self) -> u32 {
        self.runs.iter().map(|run| run.total_barters()).sum()
    }

    pub fn successful_barters(&self) -> u32 {
        self.runs.iter().map(|run| run.successful_barters()).sum()
    }

    /// The total number of pearls picked up across all runs in the stream.
    pub fn total_pearls(&self) -> u32 {
        self.runs.iter().map(|run| run.total_pearls()).sum()
    }

    /// The total number of blazes killed across all runs in the stream.
    pub fn total_fights(&self) -> u32 {
        self.runs.iter().map(|run| run.total_fights()).sum()
    }

    pub fn successful_fights(&self) -> u32 {
        self.runs.iter().map(|run| run.successful_fights()).sum()
    }

    /// The total number of blaze rods picked up across all runs in the stream.
    pub fn total_rods(&self) -> u32 {
        self.runs.iter().map(|run| run.total_rods()).sum()
    }

    /// A summary of the results of the stream.
    pub fn results(&self) -> StreamResults {
        StreamResults::new(
            &self.goals,
            self.total_barters(),
            self.total_fights(),
            self.successful_barters(),
            self.successful_fights(),
        )
    }

    /// Simulate a single run.
    fn simulate_run(
        barter_drop_sim: &mut DropSim,
        blaze_drop_sim: &mut DropSim,
        goals: &[RunGoals],
        run: usize,
    ) -> Run {
        RunSim::new(
            barter_drop_sim,
            blaze_drop_sim,
            goals[run].target_pearls,
            goals[run].target_rods,
        )
        .run()
    }
}
