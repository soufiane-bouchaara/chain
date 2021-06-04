use codec::{Decode, Encode};
use frame_support::traits::LockIdentifier;
use sp_runtime::RuntimeDebug;

pub const STAKING_ID: LockIdentifier = *b"staking ";

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
pub enum RewardDestination<AccountId> {
    /// Pay into the stash account, increasing the amount at stake accordingly.
    Staked,
    /// Pay into the stash account, not increasing the amount at stake.
    Stash,
    /// Pay into the controller account.
    Controller,
    /// Pay into a specified account.
    Account(AccountId),
    /// Receive no reward.
    None,
}

impl<AccountId> Default for RewardDestination<AccountId> {
    fn default() -> Self {
        Self::Staked
    }
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct StashLedger<AccountId, Balance> {
    /// The stash account whose balance is actually locked and at stake.
    pub stash: AccountId,
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    pub total: Balance,
    /// The total amount of the stash's balance that will be at stake in any forthcoming
    /// rounds.
    pub staked: Balance,
    /// TODO!
    pub delegated: Balance,
}

impl<AccountId, Balance> StashLedger<AccountId, Balance> {
    pub fn new(stash: AccountId, total: Balance, staked: Balance, delegated: Balance) -> Self {
        Self {
            stash,
            total,
            staked,
            delegated,
        }
    }
}
