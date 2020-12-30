use rand::rngs::ThreadRng;
use rand::Rng;

/// An item that can be part of a drop table. These are Minecraft items.
/// This list is incomplete, since it only contains the items involved in piglin barters from 1.16.1 and blaze rods.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    None,
    Book,
    IronBoots,
    Potion,
    SplashPotion,
    IronNugget,
    Quartz,
    GlowstoneDust,
    MagmaCream,
    EnderPearl,
    String,
    FireCharge,
    Gravel,
    Leather,
    MetherBrick,
    Obsidian,
    CryingObsidian,
    SoulSand,
    BlazeRod,
}

/// The configuration for a drop, but not the drop itself.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DropConfig {
    pub item: Item,
    pub weight: u32,
    pub min_count: u32,
    pub max_count: u32,
}

impl DropConfig {
    /// Creates a drop config.
    /// ```
    /// # use mc_sim::drop::*;
    /// // Create a drop config for an ender pearl.
    /// let drop_config = DropConfig::new(Item::EnderPearl, 20, 4, 8);
    /// # assert_eq!(Item::EnderPearl, drop_config.item);
    /// # assert_eq!(20, drop_config.weight);
    /// # assert_eq!(4, drop_config.min_count);
    /// # assert_eq!(8, drop_config.max_count);
    /// ```
    pub fn new(item: Item, weight: u32, min_count: u32, max_count: u32) -> Self {
        Self {
            item,
            weight,
            min_count,
            max_count,
        }
    }
}

/// An item drop. The roll is the exact roll that was made that selected this item from the drop list.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Drop {
    pub roll: u32,
    pub item: Item,
    pub count: u32,
}

/// An item drop simulator. Uses a drop list and uniform random number generation to select drops.
/// This is based on the decompiled minecraft code and I believe it is an accurate representation of that logic.
/// Some features of that code have been removed, as they don't play a part in bartering or blaze drops.
#[derive(Debug)]
pub struct DropSim {
    rng: ThreadRng,
    drop_list: Vec<DropConfig>,
    max_roll: u32,
}

impl DropSim {
    /// Creates a drop simulator.
    pub fn new(drop_list: Vec<DropConfig>) -> Self {
        let max_roll = drop_list.iter().fold(0, |sum, drop| sum + drop.weight);
        Self {
            rng: rand::thread_rng(),
            drop_list,
            max_roll,
        }
    }

    /// Gets an item drop using the drop list.
    /// ```
    /// # use mc_sim::drop::*;
    /// // Create a drop list that has a 2:1 chance to be gravel over ender pearls
    /// // and drops 4 to 8 (inclusive) ender pearls.
    /// let drop_list = vec![
    ///     DropConfig::new(Item::Gravel, 20, 8, 32),
    ///     DropConfig::new(Item::EnderPearl, 10, 4, 8),
    /// ];
    ///
    /// // Create a drop simulator for that drop list.
    /// let mut drop_sim = DropSim::new(drop_list);
    ///
    /// // Get 1000 item drops.
    /// let drops: Vec<Drop> = (0..1000).map(|_| drop_sim.get_drop()).collect();
    /// # for drop in drops {
    /// #     match drop.item {
    /// #         Item::EnderPearl => {
    /// #             assert!(drop.roll >= 21);
    /// #             assert!(drop.roll <= 30);
    /// #             assert!(drop.count >= 4);
    /// #             assert!(drop.count <= 8);
    /// #         },
    /// #         Item::Gravel => {
    /// #             assert!(drop.roll <= 20);
    /// #             assert!(drop.count >= 8);
    /// #             assert!(drop.count <= 32);
    /// #         },
    /// #         _ => assert!(false)
    /// #     };
    /// # }
    /// ```
    pub fn get_drop(&mut self) -> Drop {
        let roll: u32 = self.rng.gen_range(0..self.max_roll);
        let mut weight_remaining: i32 = roll as i32;
        let (_, item, count) = self
            .drop_list
            .iter()
            .find(|drop| {
                weight_remaining -= drop.weight as i32;
                weight_remaining <= 0
            })
            .map(|drop| {
                (
                    weight_remaining,
                    drop.item,
                    drop.min_count..=drop.max_count,
                )
            })
            .unwrap();

        Drop {
            roll,
            item,
            count: self.rng.gen_range(count),
        }
    }
}
