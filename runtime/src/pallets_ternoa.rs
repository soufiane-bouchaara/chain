use crate::{
    constants::currency::CENTS, Balances, Call, Event, Nfts, OriginCaller, Runtime, Scheduler,
    TiimeBalances, Treasury,
};
use frame_support::parameter_types;
use sp_runtime::AccountId32;
use ternoa_primitives::{AccountId, Balance, NFTId, Signature};

parameter_types! {
    pub const MintFee: Balance = 50 * CENTS;
    pub const MinimumStake: Balance = 50 * CENTS;
/*     pub CharityDest: AccountId = sp_core::sr25519::Signature::from_raw(hex_literal::hex!("5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc")); */
    //pub CharityDest: AccountId = AccountId32::from_ss58check("5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc");
    pub CharityDest: AccountId = AccountId32::from([254, 101, 113, 125, 173, 4, 71, 215, 21, 246, 96, 160, 165, 132, 17, 222, 80, 155, 66, 230, 239, 184, 55, 95, 86, 47, 88, 165, 84, 213, 134, 14]);
}

impl ternoa_nfts::Config for Runtime {
    type Event = Event;
    type NFTId = NFTId;
    type WeightInfo = ();
    type Currency = Balances;
    type MintFee = MintFee;
    type FeesCollector = Treasury;
}

impl ternoa_timed_escrow::Config for Runtime {
    type Event = Event;
    type NFTs = Nfts;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type PalletsCall = Call;
    type WeightInfo = ();
}

impl ternoa_marketplace::Config for Runtime {
    type Event = Event;
    type CurrencyCaps = Balances;
    type CurrencyTiime = TiimeBalances;
    type NFTs = Nfts;
    type WeightInfo = ();
}

impl ternoa_staking::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MinimumStake = MinimumStake;
    type CharityDest = CharityDest;
}
