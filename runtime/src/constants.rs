//! A set of constant values used in the runtime.

/// Money matters.
pub mod currency {
    use ternoa_primitives::Balance;

    pub const UNITS: Balance = 1_000_000_000_000_000_000;
    pub const EUROS: Balance = UNITS;
    pub const CENTS: Balance = UNITS / 100;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

/// Time.
pub mod time {
    use ternoa_primitives::{BlockNumber, Moment};

    /// Since BABE is probabilistic this is the average expected block time that
    /// we are targetting. Blocks will be produced at a minimum duration defined
    /// by `SLOT_DURATION`, but some slots will not be allocated to any
    /// authority and hence no block will be produced. We expect to have this
    /// block time on average following the defined slot duration and the value
    /// of `c` configured for BABE (where `1 - c` represents the probability of
    /// a slot being empty).
    /// This value is only used indirectly to define the unit constants below
    /// that are expressed in blocks. The rest of the code should use
    /// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
    /// minimum period).
    ///
    /// If using BABE with secondary slots (default) then all of the slots will
    /// always be assigned, in which case `MILLISECS_PER_BLOCK` and
    /// `SLOT_DURATION` should have the same value.
    ///
    /// <https://research.web3.foundation/en/latest/polkadot/block-production/Babe.html#-6.-practical-results>
    pub const MILLISECS_PER_BLOCK: Moment = 6000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
    pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = 1 * HOURS;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
    pub const WEEKS: BlockNumber = DAYS * 7;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}
