use crate::drop::{DropConfig, Item};
use crate::error::McSimError;
use cached::proc_macro::cached;
use fraction::BigUint;
use fraction::Zero;
use statrs::distribution::{Discrete, NegativeBinomial, Univariate};
type F = fraction::GenericFraction<BigUint>;

#[derive(Debug, Clone, Copy)]
pub struct EnderPearlDistribution {
    ender_pearl_target_total: u32,
    ender_pearl_target_per_run: u32,
    distribution: NegativeBinomial,
}

impl EnderPearlDistribution {
    /// Creates a distribution that has a probability mass function that represents the number of expected
    /// failed barters (trades where you did not get pearls) given the number of pearls targeted over a "stream"
    /// and the average target pearl drops in that stream.
    ///
    /// This distribution is most accurate when all runs within the stream have the same target pearl drops.
    /// Deviating from that by much creates an offset in the data, that provides an unrealistic favourability towards
    /// good luck typically.
    ///
    /// When constrained in this way, this distribution is fairly accurate.
    /// For this reason, I suggest that you feed in modified data rather than the exact run data from dream's streams.
    /// I.E. I took all runs that made it beyond 10 pearls (17 runs) and then set the target to 10 across the board.
    /// This is what I consider to be the most fair to Dream, since 10 pearls is a typical target number and the exact
    /// number you get to isn't relevant. Now, you can simply ask how many trades it took to arrive at 10 pearls
    /// for those 17 streams, and then compare that with this distribution to get "Dream's luck".
    ///
    /// Alternatively, you can feed in exact run data, however, put each run in a separate "stream" in the simulator.
    /// Then, average the results over all streams.
    /// ```
    /// # use mc_sim::drop_list;
    /// // Estimates the probability of Dream's luck for his 17 runs that got 10+ pearls.
    /// // It is not possible from the vods to know exactly how many barters it took dream
    /// // to get these results, so I have taken the worst case number of barters (239),
    /// // which makes this test *highly* favoured towards dream.
    /// // favoured number of 235 required barters.
    /// let number_of_runs = 17;
    /// let barters_made = 239;
    /// let successful_barters = 39;
    /// let ender_pearl_target_per_run = 10;
    /// let ender_pearl_target_total = ender_pearl_target_per_run * number_of_runs;
    ///
    /// let drop_list = drop_list::barter_drop_list(ender_pearl_target_total, ender_pearl_target_per_run);
    /// let probability_of_dream_luck = drop_list.distribution().unwrap().luck(barters_made, successful_barters);
    ///
    /// // This produces a slightly different number than the mods paper, because I'm simulating trades where
    /// // the number of pearls dropped is also a variable. Thus, I am not asking the same question they did.
    /// assert_eq!(probability_of_dream_luck, 0.0000000006713608557973316);
    /// ```
    pub fn new(
        ender_pearl_target_total: u32,
        ender_pearl_target_per_run: u32,
        drop_list: &[DropConfig],
    ) -> Result<Self, McSimError> {
        EnderPearlDistribution::create_distribution(
            ender_pearl_target_total,
            ender_pearl_target_per_run,
            drop_list,
        )
        .map(|distribution| Self {
            ender_pearl_target_total,
            ender_pearl_target_per_run,
            distribution,
        })
    }

    /// Gets the negative binomial distribution for ender pearls, for the target number of pearls in total and per run.
    pub fn distribution(&self) -> &NegativeBinomial {
        &self.distribution
    }

    /// An estimate of the luck of the total number of barters and number of successful barters resulting
    /// in the target number of ender pearls, based on this distribution.
    pub fn luck(&self, total_barters_made: u32, successful_barters: u32) -> f64 {
        self.distribution
            .cdf(total_barters_made as f64 - successful_barters as f64)
    }

    /// An estimate of the probability of the specific total number of barters and number of successful barters resulting
    /// in the target number of ender pearls, based on this distribution.
    pub fn probability(&self, total_barters_made: u32, successful_barters: u32) -> f64 {
        self.distribution
            .pmf((total_barters_made as i32 - successful_barters as i32) as u64)
    }

    /// Creates the actual distribution.
    /// Described in the documentation for [new](EnderPearlDistribution::new).
    fn create_distribution(
        ender_pearl_target_total: u32,
        ender_pearl_target_per_run: u32,
        drop_list: &[DropConfig],
    ) -> Result<NegativeBinomial, McSimError> {
        let drop_probability = item_drop_probability(drop_list, Item::EnderPearl);
        let drop_range = item_drop_range(drop_list, Item::EnderPearl);

        let mean_drops_to_reach_target = attempts_to_reach_target(
            drop_range.0 as i32,
            drop_range.1 as i32,
            ender_pearl_target_per_run as i32,
        );

        NegativeBinomial::new(
            ender_pearl_target_total as f64 / ender_pearl_target_per_run as f64
                * mean_drops_to_reach_target,
            drop_probability,
        )
        .map_err(|_| McSimError::InvalidDistribution)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlazeRodDistribution {
    blaze_rod_target: u32,
    distribution: NegativeBinomial,
}

impl BlazeRodDistribution {
    /// Creates a distribution that has a probability mass function that represents the number of expected
    /// failures for a given number of successful blaze fights where a blaze rod was dropped.
    ///
    /// This distribution is highly accurate and copes with variable numbers of blaze rod targets per run in a stream.
    /// This is because 1 blaze rod drop = 1 successful blaze rod fight, making negative binomial highly applicable.
    /// For this reason, I suggest that you feed the exact data of dreams runs into a single "stream" in the simulation.
    /// The resulting expected probability curve will closely align with data samples from simulation runs.
    /// ```
    /// # use mc_sim::drop_list;
    /// // Estimates the probability of Dream's luck for his 22 runs of getting 211 blaze rods (or more)
    /// // within the number of blazes he killed (305).
    /// let blazes_killed = 305;
    /// let number_of_rods = 211;
    ///
    /// let drop_list = drop_list::blaze_drop_list(number_of_rods);
    /// let probability_of_dream_luck = drop_list.distribution().unwrap().luck(blazes_killed);
    ///
    /// // This produces the same number calculated in the mods paper.
    /// assert_eq!(probability_of_dream_luck, 0.000000000008791412042796765);
    /// ```
    pub fn new(blaze_rod_target: u32, drop_list: &[DropConfig]) -> Result<Self, McSimError> {
        BlazeRodDistribution::create_distribution(blaze_rod_target, drop_list).map(|distribution| {
            Self {
                blaze_rod_target,
                distribution,
            }
        })
    }

    /// Gets the negative binomial distribution for ender pearls, for the target number of pearls in total and per run.
    pub fn distribution(&self) -> &NegativeBinomial {
        &self.distribution
    }

    /// An estimate of the luck of the number of blazes killed to obtain the target number of blaze rods,
    /// based on this distribution.
    pub fn luck(&self, total_blazes_killed: u32) -> f64 {
        self.distribution
            .cdf(total_blazes_killed as f64 - self.blaze_rod_target as f64)
    }

    /// An estimate of the probability of the specific number of blazes killed to obtain the target number of blaze rods,
    /// based on this distribution.
    pub fn probability(&self, total_blazes_killed: u32) -> f64 {
        self.distribution
            .pmf((total_blazes_killed as i32 - self.blaze_rod_target as i32) as u64)
    }

    /// Creates the actual distribution.
    /// Described in the documentation for [new](BlazeRodDistribution::new).
    fn create_distribution(
        blaze_rod_target: u32,
        drop_list: &[DropConfig],
    ) -> Result<NegativeBinomial, McSimError> {
        NegativeBinomial::new(
            blaze_rod_target as f64,
            item_drop_average(drop_list, Item::BlazeRod),
        )
        .map_err(|_| McSimError::InvalidDistribution)
    }
}

/// Computes the mean probability of getting a specific item drop from a drop list.
/// Assumes that the drop list only has the item once in the list.
/// ```
/// # use mc_sim::drop::Item;
/// # use mc_sim::drop_list;
/// # use mc_sim::stats;
/// assert_eq!(stats::item_drop_probability(drop_list::blaze_drop_list(7).list(), Item::BlazeRod), 1.0);
/// assert_eq!(stats::item_drop_probability(drop_list::barter_drop_list(10, 10).list(), Item::EnderPearl), 20.0 / 423.0);
/// ```
pub fn item_drop_probability(drop_list: &[DropConfig], item: Item) -> f64 {
    let target = drop_list.iter().find(|d| d.item == item).unwrap();
    target.weight as f64 / drop_list.iter().map(|d| d.weight as f64).sum::<f64>()
}

/// Computes the mean number of items dropped for a given item on a drop list.
/// Assumes that the drop list only has the item once in the list.
/// ```
/// # use mc_sim::drop::Item;
/// # use mc_sim::drop_list;
/// # use mc_sim::stats;
/// assert_eq!(stats::item_drop_average(drop_list::blaze_drop_list(7).list(), Item::BlazeRod), 0.5);
/// assert_eq!(stats::item_drop_average(drop_list::barter_drop_list(10, 10).list(), Item::EnderPearl), 6.0);
/// ```
pub fn item_drop_average(drop_list: &[DropConfig], item: Item) -> f64 {
    let target = drop_list.iter().find(|d| d.item == item).unwrap();
    (target.max_count as f64 - target.min_count as f64) / 2.0 + target.min_count as f64
}

/// Provides the minimum and maximum amount that a drop of an item could provide from a drop list.
/// Assumes that the drop list only has the item once in the list.
/// ```
/// # use mc_sim::drop::Item;
/// # use mc_sim::drop_list;
/// # use mc_sim::stats;
/// assert_eq!(stats::item_drop_range(drop_list::blaze_drop_list(7).list(), Item::BlazeRod), (0, 1));
/// assert_eq!(stats::item_drop_range(drop_list::barter_drop_list(10, 10).list(), Item::EnderPearl), (4, 8));
/// ```
pub fn item_drop_range(drop_list: &[DropConfig], item: Item) -> (u32, u32) {
    let target = drop_list.iter().find(|d| d.item == item).unwrap();
    (target.min_count, target.max_count)
}

/// Answers the question "how many dice do I need to roll to get to a target"?
/// Implementation based on the answer by Varun Vejalla: [https://math.stackexchange.com/a/3965269/867664](https://math.stackexchange.com/a/3965269/867664)
/// ```
/// # use mc_sim::drop::Item;
/// # use mc_sim::drop_list;
/// # use mc_sim::stats;
/// assert_eq!(round(stats::attempts_to_reach_target(1, 6, 1), 4), 1.0000);
/// assert_eq!(round(stats::attempts_to_reach_target(1, 6, 4), 4), 1.5880);
/// assert_eq!(round(stats::attempts_to_reach_target(1, 6, 30), 4), 9.0476);
/// assert_eq!(round(stats::attempts_to_reach_target(1, 6, 36), 4), 10.7619);
/// assert_eq!(round(stats::attempts_to_reach_target(1, 6, 80), 4), 23.3333);
///
/// let drop_list = drop_list::barter_drop_list(10, 10);
/// let drop_range = stats::item_drop_range(drop_list.list(), Item::EnderPearl);
/// assert_eq!(
///     round(
///         stats::attempts_to_reach_target(drop_range.0 as i32, drop_range.1 as i32, 10),
///         4
///     ),
///     2.1200
/// );
///
/// fn round(f: f64, p: u32) -> f64 {
///     let precision = (10.0 as f64).powf(p as f64);
///     (f * precision).round() / precision
/// }
/// ```
pub fn attempts_to_reach_target(min: i32, max: i32, target: i32) -> f64 {
    attempts_to_reach_target_cached(min, max, target)
}

#[cached]
fn attempts_to_reach_target_cached(min: i32, max: i32, target: i32) -> f64 {
    match target {
        _ if target <= 0 => 0.0,
        _ => {
            1.0 + 1.0 / (max - min + 1) as f64
                * (min..(max + 1))
                    .map(|k| attempts_to_reach_target_cached(min, max, target - k))
                    .sum::<f64>()
        }
    }
}

/// This struct implements the answer to the problem of "how many dice do I need to roll to get to a target"
/// provided by user Tomáš Hons: [https://math.stackexchange.com/a/3965202/867664](https://math.stackexchange.com/a/3965202/867664)
/// Ultimately, this provides the same answer as the much simpler implementation
/// in [attempts_to_reach_target] which should be used instead.
pub struct UniformProbabilityTable {
    samples: usize,
    distribution_size: usize,
    table: Vec<Vec<F>>,
}

impl UniformProbabilityTable {
    pub fn generate(samples: u32, distribution_size: u32) -> Self {
        let samples = samples as usize;
        let distribution_size = distribution_size as usize;

        Self {
            samples,
            distribution_size,
            table: UniformProbabilityTable::uniform_probabilities_for_n_samples(
                samples,
                distribution_size,
            ),
        }
    }

    pub fn expectation_of_target(&self) -> F {
        let probabilities = self.probabilities();
        let expectations = self.expectations();

        let mut normalization_factor = F::zero();
        let mut exp_contrib = F::zero();

        for min_dots in 0..self.distribution_size {
            let prob = F::new(
                (self.distribution_size - min_dots) as u64,
                self.distribution_size as u64,
            ) * probabilities[self.samples - min_dots - 1].clone();
            normalization_factor += prob.clone();
            exp_contrib += prob * (expectations[self.samples - min_dots - 1].clone() + F::from(1));
        }

        exp_contrib / normalization_factor
    }

    fn expectations(&self) -> Vec<F> {
        let probabilities = self.probabilities();
        let mut expectations = vec![F::zero()];

        for num in 1..self.samples {
            let mut normalization_factor = F::zero();
            let mut exp_contrib = F::zero();

            for dots in 1..(std::cmp::min(self.distribution_size, num) + 1) {
                let prob = probabilities[num - dots].clone();
                normalization_factor += prob.clone();
                exp_contrib += prob * (expectations[num - dots].clone() + F::from(1));
            }

            expectations.push(exp_contrib / normalization_factor);
        }

        expectations
    }

    fn probabilities(&self) -> Vec<F> {
        (0..self.samples)
            .map(|num| self.probability_of_number(num))
            .collect()
    }

    fn probability_of_number(&self, num: usize) -> F {
        (0..self.samples)
            .map(|throw| self.table[throw][num].clone())
            .sum()
    }

    fn uniform_probabilities_for_n_samples(
        samples: usize,
        distribution_size: usize,
    ) -> Vec<Vec<F>> {
        let mut probabilities =
            UniformProbabilityTable::create_uniform_probabilities_table(samples);

        for throw in 1..samples {
            for num in 1..samples {
                probabilities[throw][num] = (1..std::cmp::min(num + 1, distribution_size + 1))
                    .map(|dots| probabilities[throw - 1][num - dots].clone())
                    .sum::<F>()
                    / F::from(distribution_size)
            }
        }

        probabilities
    }

    fn create_uniform_probabilities_table(size: usize) -> Vec<Vec<F>> {
        let mut table: Vec<Vec<F>> = (0..size)
            .map(|_| (0..size).map(|_| F::from(0.0)).collect())
            .collect();
        table[0][0] = F::from(1.0);
        table
    }
}
