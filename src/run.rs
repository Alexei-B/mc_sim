use crate::drop::{Drop, DropSim, Item};

/// Represents a single speed run, in which barters are made and blazes are fought.
/// The results of bartering and fighting are stored as a list of drops that can be interrogated
/// to see exactly how lucky or unlucky the run was.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Run {
    pub barters: Vec<Drop>,
    pub fights: Vec<Drop>,
}

impl Run {
    /// Create a run from the results of bartering with piglins and fighting blazes.
    /// ```
    /// # use mc_sim::drop::*;
    /// # use mc_sim::run::*;
    /// let barters = vec![
    ///     Drop { item: Item::Gravel, roll: 0, count: 1 },
    ///     Drop { item: Item::Gravel, roll: 0, count: 1 },
    ///     Drop { item: Item::EnderPearl, roll: 0, count: 1 },
    ///     Drop { item: Item::Gravel, roll: 0, count: 1 },
    ///     Drop { item: Item::EnderPearl, roll: 0, count: 3 },
    /// ];
    ///
    /// let fights = vec![
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 0 },
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 0 },
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 1 },
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 0 },
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 0 },
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 1 },
    ///     Drop { item: Item::BlazeRod, roll: 0, count: 1 },
    /// ];
    ///
    /// let run = Run::new(barters, fights);
    /// assert_eq!(run.total_barters(), 5);
    /// assert_eq!(run.total_pearls(), 4);
    /// assert_eq!(run.total_fights(), 7);
    /// assert_eq!(run.total_rods(), 3);
    /// ```
    pub fn new(barters: Vec<Drop>, fights: Vec<Drop>) -> Self {
        Self { barters, fights }
    }

    /// The total number of barters that were made in the run.
    pub fn total_barters(&self) -> u32 {
        self.barters.len() as u32
    }

    pub fn successful_barters(&self) -> u32 {
        self.barters
            .iter()
            .filter(|drop| drop.item == Item::EnderPearl)
            .count() as u32
    }

    /// The total number of pearls that were obtained during the run.
    pub fn total_pearls(&self) -> u32 {
        self.barters
            .iter()
            .filter(|drop| drop.item == Item::EnderPearl)
            .map(|drop| drop.count)
            .sum()
    }

    /// The total number of blazes that were killed in the run.
    pub fn total_fights(&self) -> u32 {
        self.fights.len() as u32
    }

    pub fn successful_fights(&self) -> u32 {
        self.barters
            .iter()
            .filter(|drop| drop.item == Item::BlazeRod)
            .count() as u32
    }

    /// The total number of blaze rods that were obtained during the run.
    pub fn total_rods(&self) -> u32 {
        self.fights
            .iter()
            .filter(|drop| drop.item == Item::BlazeRod)
            .map(|drop| drop.count)
            .sum()
    }
}

/// The goals of a run simulation.
/// This represents the minimum resources a runner is looking for out of this run before moving on.
/// E.G. total_pearls is the number of ender pearls the runner wants before they stop trading with piglins.
///
/// This does not take into account ideas like "batches" of trades, where a runner might choose to leave
/// before reaching their goal because the run won't pb if they have to trade any more and they just hope
/// that they get good portal luck.
///
/// Ideas like this are not in scope for this simulation and can be accounted for in the analysis of the data.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct RunGoals {
    pub target_pearls: u32,
    pub target_rods: u32,
}

/// A Minecraft speed run simulation.
#[derive(Debug)]
pub struct RunSim<'a, 'b> {
    barter_drop_sim: &'a mut DropSim,
    blaze_drop_sim: &'b mut DropSim,
    pearl_target: u32,
    rods_target: u32,
}

impl<'a, 'b> RunSim<'a, 'b> {
    /// Creates a minecraft speed run simulator.
    /// ```
    /// # use mc_sim::drop::*;
    /// # use mc_sim::drop_list;
    /// # use mc_sim::run::*;
    /// let mut barter_drop_sim = DropSim::new(drop_list::barter_drop_list(10, 10).list_clone());
    /// let mut blaze_drop_sim = DropSim::new(drop_list::blaze_drop_list(7).list_clone());
    ///
    /// let mut run_sim = RunSim::new(&mut barter_drop_sim, &mut blaze_drop_sim, 10, 7);
    /// let run = run_sim.run();
    /// assert!(run.total_pearls() >= 10);
    /// assert!(run.total_rods() >= 7);
    /// ```
    pub fn new(
        barter_drop_sim: &'a mut DropSim,
        blaze_drop_sim: &'b mut DropSim,
        pearl_target: u32,
        rods_target: u32,
    ) -> Self {
        Self {
            barter_drop_sim,
            blaze_drop_sim,
            pearl_target,
            rods_target,
        }
    }

    /// Simulate a run.
    pub fn run(&mut self) -> Run {
        Run::new(self.barter_for_pearls(), self.fight_for_rods())
    }

    /// Barter for pearls until the pearl target is reached.
    pub fn barter_for_pearls(&mut self) -> Vec<Drop> {
        RunSim::farm_for_item(
            &mut self.barter_drop_sim,
            Item::EnderPearl,
            self.pearl_target,
        )
    }

    /// Fight blazes until the rod target is reached.
    pub fn fight_for_rods(&mut self) -> Vec<Drop> {
        RunSim::farm_for_item(&mut self.blaze_drop_sim, Item::BlazeRod, self.rods_target)
    }

    /// Farm for an item from a drop simulator with a minimum target before we're done.
    pub fn farm_for_item(drop_sim: &mut DropSim, item: Item, minimum: u32) -> Vec<Drop> {
        let mut drops = Vec::new();
        let mut count = 0;

        while count < minimum {
            let drop = drop_sim.get_drop();

            if drop.item == item {
                count += drop.count;
            }

            drops.push(drop);
        }

        drops
    }
}
