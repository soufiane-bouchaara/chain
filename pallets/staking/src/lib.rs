#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod types;

pub use pallet::*;
pub use types::*;

use frame_support::traits::{Currency, Get, LockableCurrency, WithdrawReasons};
use sp_runtime::traits::Zero;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{ensure, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{CheckedSub, StaticLookup};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// TODO!
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
        /// TODO!
        #[pallet::constant]
        type MinimumStake: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        //
        //  Privileged
        //

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn change_permanent_validator(
            origin: OriginFor<T>,
            account: <T::Lookup as StaticLookup>::Source,
            is_permanent: bool,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let account = T::Lookup::lookup(account)?;

            if is_permanent {
                InvulnerableValidators::<T>::mutate(|permanents| {
                    if !permanents.contains(&account) {
                        permanents.push(account);
                    }
                });
            } else {
                InvulnerableValidators::<T>::mutate(|permanents| {
                    let index = permanents.iter().position(|x| *x == account);
                    if let Some(index) = index {
                        // TODO! Maybe swap_remove?
                        permanents.remove(index);
                    }
                });
            }

            Ok(().into())
        }

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn set_maximum_validator_count(
            origin: OriginFor<T>,
            count: u32,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            MaximumValidatorCount::<T>::put(count);

            Ok(().into())
        }

        //
        // Permission-less
        //

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn bond(
            origin: OriginFor<T>,
            controller: <T::Lookup as StaticLookup>::Source,
            amount: BalanceOf<T>,
            reward_destination: RewardDestination<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let stash = ensure_signed(origin)?;

            if Bonded::<T>::contains_key(&stash) {
                return Err(Error::<T>::AlreadyBonded.into());
            }

            if amount < T::MinimumStake::get() {
                return Err(Error::<T>::InsufficientValue.into());
            }

            if T::Currency::free_balance(&stash) < amount {
                return Err(Error::<T>::InsufficientFunds.into());
            }

            let controller = T::Lookup::lookup(controller)?;
            Bonded::<T>::insert(&stash, (&controller, reward_destination));

            let ledger = StashLedger::new(stash, amount, amount, Zero::zero());
            Self::update_ledger(&controller, &ledger);

            // TODO!
            Ok(().into())
        }

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn bond_extra(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let stash = ensure_signed(origin)?;

            let controller = Bonded::<T>::get(&stash).ok_or(Error::<T>::NotStash)?.0;
            let mut ledger = Ledger::<T>::get(&controller).ok_or(Error::<T>::NotController)?;

            let stash_balance = T::Currency::free_balance(&stash);
            if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
                let extra = extra.min(amount);
                ledger.total += extra;
                ledger.staked += extra;
                // TODO! last check: the new active amount of ledger must be more than ED.
                ensure!(
                    ledger.staked >= T::Currency::minimum_balance(),
                    Error::<T>::InsufficientValue
                );

                Self::update_ledger(&controller, &ledger);
            }

            // TODO!
            Ok(().into())
        }

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn unbond(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let controller = ensure_signed(origin)?;
            let mut ledger = Ledger::<T>::get(&controller).ok_or(Error::<T>::NotController)?;

            let mut amount = amount.min(ledger.staked);
            if amount.is_zero() {
                return Ok(().into());
            }

            ledger.staked -= amount;

            if ledger.staked < T::MinimumStake::get() {
                amount += ledger.staked;
                ledger.staked = Zero::zero();
            }

            Self::update_ledger(&controller, &ledger);

            // TODO!
            Ok(().into())
        }

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn delegate(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            delegator: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let caller_stash = ensure_signed(origin)?;
            let delegator_stash = T::Lookup::lookup(delegator)?;

            // Making sure that the delegator exists
            let controller = Bonded::<T>::get(&delegator_stash)
                .ok_or(Error::<T>::NotStash)?
                .0;
            ensure!(
                Ledger::<T>::contains_key(&controller),
                Error::<T>::NotController
            );

            // Making sure the caller is OK
            if T::Currency::free_balance(&caller_stash) < amount {
                return Err(Error::<T>::InsufficientFunds.into());
            }

            let controller = Bonded::<T>::get(&caller_stash)
                .ok_or(Error::<T>::NotStash)?
                .0;
            let mut ledger = Ledger::<T>::get(&controller).ok_or(Error::<T>::NotController)?;

            ledger.total += amount;
            ledger.delegated += amount;

            Self::update_ledger(&controller, &ledger);
            Delegated::<T>::mutate(&controller, |x| {
                if let Some(balances) = x {
                    if let Some(balance) = balances.get_mut(&delegator_stash) {
                        *balance += amount;
                    } else {
                        balances.insert(delegator_stash, amount);
                    }
                } else {
                    let mut map = BTreeMap::new();
                    map.insert(delegator_stash, amount);
                    *x = Some(map);
                }
            });

            // TODO!
            Ok(().into())
        }

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn undelegate(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            delegator: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let caller_stash = ensure_signed(origin)?;
            let delegator_stash = T::Lookup::lookup(delegator)?;

            // Making sure that the delegator exists
            let controller = Bonded::<T>::get(&delegator_stash)
                .ok_or(Error::<T>::NotStash)?
                .0;
            ensure!(
                Ledger::<T>::contains_key(&controller),
                Error::<T>::NotController
            );

            // Making sure the caller is OK
            let controller = Bonded::<T>::get(&caller_stash)
                .ok_or(Error::<T>::NotStash)?
                .0;
            let mut ledger = Ledger::<T>::get(&controller).ok_or(Error::<T>::NotController)?;

            let mut delegator_balances =
                Delegated::<T>::get(&controller).ok_or(Error::<T>::NotController)?;

            let balance = delegator_balances
                .get_mut(&delegator_stash)
                .ok_or(Error::<T>::InsufficientFunds)?;

            let amount = amount.min(*balance);
            ensure!(ledger.delegated >= amount, Error::<T>::InsufficientFunds);

            *balance -= amount;

            ledger.total -= amount;
            ledger.delegated -= amount;

            Delegated::<T>::insert(&controller, delegator_balances);
            Self::update_ledger(&controller, &ledger);

            // TODO!
            Ok(().into())
        }

        /// TODO!
        #[pallet::weight(1_000_000)]
        pub fn claim(
            origin: OriginFor<T>,
            session_id: u32,
            validator_account: <T::Lookup as StaticLookup>::Source,
            delegator_account: Option<<T::Lookup as StaticLookup>::Source>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // TODO!
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::NFTId = "NFTId")]
    pub enum Event<T: Config> {
        /// A new NFT was created. \[nft id, owner, series id\]
        Created(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// TODO!
        InsufficientValue,
        /// TODO!
        AlreadyBonded,
        /// TODO!
        InsufficientFunds,
        /// TODO!
        NotStash,
        /// TODO!
        NotController,
        /// TODO!
        InvalidValue,
    }

    /// TODO!
    #[pallet::storage]
    #[pallet::getter(fn maximum_validator_count)]
    pub type MaximumValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    // TODO! Whitelist - internal
    #[pallet::storage]
    #[pallet::getter(fn permanent_validators)]
    pub type InvulnerableValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bonded)]
    pub type Bonded<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (T::AccountId, RewardDestination<T::AccountId>),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub type Ledger<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        StashLedger<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn delegated)]
    pub type Delegated<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BTreeMap<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    /// TODO!
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub maximum_validator_count: u32,
        phantom: PhantomData<T>,
    }

    /// TODO!
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                maximum_validator_count: Default::default(),
                phantom: PhantomData,
            }
        }
    }

    /// TODO!
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            MaximumValidatorCount::<T>::put(self.maximum_validator_count);
        }
    }
}

impl<T: Config> Pallet<T> {
    fn update_ledger(controller: &T::AccountId, ledger: &StashLedger<T::AccountId, BalanceOf<T>>) {
        T::Currency::set_lock(
            STAKING_ID,
            &ledger.stash,
            ledger.total,
            WithdrawReasons::RESERVE,
        );
        Ledger::<T>::insert(controller, ledger);
    }
}
