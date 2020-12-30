use crate::drop::{DropConfig, Item};
use crate::error::McSimError;
use crate::stats::{BlazeRodDistribution, EnderPearlDistribution};

/// Holds a list of drops and a model of the distribution of those drops.
/// See: [barter_drop_list] and [blaze_drop_list]
#[derive(Debug, Clone)]
pub struct DropList<D>
where
    D: Clone,
{
    list: Vec<DropConfig>,
    distribution: Option<D>,
}

impl<D> DropList<D>
where
    D: Clone,
{
    /// Creates a drop list with a distribution.
    fn new(list: Vec<DropConfig>, distribution: Result<D, McSimError>) -> DropList<D> {
        DropList { list, distribution: distribution.map(|d| Some(d)).unwrap_or(None) }
    }

    /// The list of drop configs, used by drop sims to pick what item to drop.
    pub fn list(&self) -> &[DropConfig] {
        &self.list
    }

    /// A clone of the list of drop configs, used by drop sims to pick what item to drop.
    pub fn list_clone(&self) -> Vec<DropConfig> {
        self.list.clone()
    }

    /// The distribution for the drops in this drop list.
    pub fn distribution(&self) -> &Option<D> {
        &self.distribution
    }
}

/// The drop list for piglin barters in Minecraft 1.16.1
pub fn barter_drop_list(
    ender_pearl_target_total: u32,
    ender_pearl_target_per_run: u32,
) -> DropList<EnderPearlDistribution> {
    let list = vec![
        DropConfig::new(Item::Book, 5, 1, 1),
        DropConfig::new(Item::IronBoots, 8, 1, 1),
        DropConfig::new(Item::Potion, 10, 1, 1),
        DropConfig::new(Item::SplashPotion, 10, 1, 1),
        DropConfig::new(Item::IronNugget, 10, 9, 36),
        DropConfig::new(Item::Quartz, 20, 8, 16),
        DropConfig::new(Item::GlowstoneDust, 20, 5, 12),
        DropConfig::new(Item::MagmaCream, 20, 2, 6),
        DropConfig::new(Item::EnderPearl, 20, 4, 8),
        DropConfig::new(Item::String, 20, 8, 24),
        DropConfig::new(Item::FireCharge, 40, 1, 5),
        DropConfig::new(Item::Gravel, 40, 8, 16),
        DropConfig::new(Item::Leather, 40, 4, 10),
        DropConfig::new(Item::MetherBrick, 40, 4, 16),
        DropConfig::new(Item::Obsidian, 40, 1, 1),
        DropConfig::new(Item::CryingObsidian, 40, 1, 3),
        DropConfig::new(Item::SoulSand, 40, 4, 16),
    ];

    let distribution =
        EnderPearlDistribution::new(ender_pearl_target_total, ender_pearl_target_per_run, &list);

    DropList::new(list, distribution)
}

/// The drop list for blaze fights in Minecraft 1.16.1
pub fn blaze_drop_list(blaze_rod_target: u32) -> DropList<BlazeRodDistribution> {
    let list = vec![DropConfig::new(Item::BlazeRod, 1, 0, 1)];
    let distribution = BlazeRodDistribution::new(blaze_rod_target, &list);

    DropList::new(list, distribution)
}
