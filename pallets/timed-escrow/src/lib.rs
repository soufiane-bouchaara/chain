#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod default_weights;
#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::traits::LockIdentifier;
use frame_support::weights::Weight;
use sp_std::vec;
use sp_std::vec::Vec;

/// Used for derivating scheduled tasks IDs
const ESCROW_ID: LockIdentifier = *b"escrow  ";

pub trait WeightInfo {
    fn create() -> Weight;
    fn cancel() -> Weight;
    fn complete_transfer() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::schedule::{DispatchTime, Named as ScheduleNamed};
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    use sp_runtime::traits::{Dispatchable, StaticLookup};
    use ternoa_common::traits::{LockableNFTs, NFTs};

    pub type NFTIdOf<T> = <<T as Config>::NFTs as LockableNFTs>::NFTId;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Pallet managing NFTs.
        type NFTs: LockableNFTs<AccountId = Self::AccountId>
            + NFTs<AccountId = Self::AccountId, NFTId = NFTIdOf<Self>>;
        /// Scheduler instance which we use to schedule actual transfer calls. This way, we have
        /// all scheduled calls accross all pallets in one place.
        type Scheduler: ScheduleNamed<Self::BlockNumber, Self::PalletsCall, Self::PalletsOrigin>;
        /// Overarching type of all pallets origins. Used with the scheduler.
        type PalletsOrigin: From<RawOrigin<Self::AccountId>>;
        /// Overarching type of all pallets calls. Used by the scheduler.
        type PalletsCall: Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
        /// Weight values for this pallet
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a timed transfer. This will lock the associated capsule until it gets
        /// transferred or canceled.
        #[pallet::weight(T::WeightInfo::create())]
        pub fn create(
            origin: OriginFor<T>,
            nft_id: NFTIdOf<T>,
            to: Vec<<T::Lookup as StaticLookup>::Source>,
            at: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(T::NFTs::owner(nft_id) == who, Error::<T>::NotNFTOwner);
            ensure!(!to.is_empty(), Error::<T>::AddressesCannotBeBlank);

            let mut lookups = vec![];
            for source in to {
                lookups.push(T::Lookup::lookup(source)?);
            }

            T::NFTs::lock(nft_id)?;

            let id = (ESCROW_ID, nft_id).encode();
            let when = DispatchTime::At(at);
            let origin = RawOrigin::Root.into();
            let call = Call::complete_transfer(lookups.clone(), nft_id).into();
            // priority was chosen arbitrarily, we made sure it is lower than runtime
            // upgrades and democracy calls
            ensure!(
                T::Scheduler::schedule_named(id, when, None, 100, origin, call).is_ok(),
                Error::<T>::SchedulingFailed
            );

            Self::deposit_event(Event::TransferScheduled(nft_id, lookups, at));

            Ok(().into())
        }

        /// Cancel a transfer that was previously created and unlocks the capsule.
        #[pallet::weight(T::WeightInfo::cancel())]
        pub fn cancel(origin: OriginFor<T>, nft_id: NFTIdOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(T::NFTs::owner(nft_id) == who, Error::<T>::NotNFTOwner);

            ensure!(
                T::Scheduler::cancel_named((ESCROW_ID, nft_id).encode()).is_ok(),
                Error::<T>::SchedulingFailed
            );
            T::NFTs::unlock(nft_id);

            Self::deposit_event(Event::TransferCanceled(nft_id));

            Ok(().into())
        }

        /// System only. Execute a transfer, called by the scheduler.
        #[pallet::weight(T::WeightInfo::complete_transfer())]
        pub fn complete_transfer(
            origin: OriginFor<T>,
            to: Vec<T::AccountId>,
            nft_id: NFTIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // We do not verify anything else as the only way for this function
            // to be called is if it was scheduled via either root action (trusted)
            // or the call to `create` which will verify NFT ownership and locking
            // status.
            ensure_root(origin)?;
            T::NFTs::unlock(nft_id);
            if to.len() == 1 {
                T::NFTs::set_owner(nft_id, &to[0])?;
            } else {
                let owner = T::NFTs::owner(nft_id);
                let details = T::NFTs::details(nft_id);

                T::NFTs::set_owner(nft_id, &to[0])?;

                for account in to.iter().skip(1) {
                    let new_nft = T::NFTs::create(&owner, details.clone()).unwrap();
                    T::NFTs::set_owner(new_nft, account)?;
                }
            }

            Self::deposit_event(Event::TransferCompleted(nft_id));

            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::BlockNumber = "BlockNumber",
        NFTIdOf<T> = "NFTId"
    )]
    pub enum Event<T: Config> {
        /// A transfer has been scheduled. \[capsule id, destination, block of transfer\]
        TransferScheduled(NFTIdOf<T>, Vec<T::AccountId>, T::BlockNumber),
        /// A transfer has been canceled. \[capsule id\]
        TransferCanceled(NFTIdOf<T>),
        /// A transfer was executed and finalized. \[capsule id\]
        TransferCompleted(NFTIdOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// This function is reserved to the owner of a nft.
        NotNFTOwner,
        /// An unknown error happened which made the scheduling call fail.
        SchedulingFailed,
        /// An unknown error happened which made the scheduling call fail.
        AddressesCannotBeBlank,
    }
}
