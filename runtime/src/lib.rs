//! Ternoa Chain runtime. This puts all the pallets together and gets compiled
//! to WASM.

// STD is not available in WASM.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use frame_support::{construct_runtime, traits::KeyOwnerProofSystem};
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, NumberFor};
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidity};
use sp_runtime::{generic, ApplyExtrinsicResult};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;
use ternoa_primitives::{AccountId, Balance, BlockNumber, Index, Signature};

pub mod constants;
mod pallets;
mod version;
mod voter_bags;

pub use pallet_balances::Call as BalancesCall;
use pallets::EpochDuration;
pub use pallets::MaxNominations as MAX_NOMINATIONS;
pub use pallets::SessionKeys;
pub use pallets::BABE_GENESIS_EPOCH_CONFIG;
pub use version::VERSION;

#[cfg(feature = "std")]
pub use version::native_version;

#[cfg(any(feature = "std", test))]
pub use pallets::StakerStatus;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is built with \
        `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
        the flag disabled.",
    )
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = ternoa_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff; balances is uncallable initially
        System: frame_system = 0,

         // Babe must be before session.
        Babe: pallet_babe = 1,

        Timestamp: pallet_timestamp = 2,
        Indices: pallet_indices = 3,
        Balances: pallet_balances = 4,
        TransactionPayment: pallet_transaction_payment = 5,

        // Consensus support.
        // Authorship must be before session in order to note author in the correct session and era
        // for im-online and staking.
        Authorship: pallet_authorship,
        Staking: pallet_staking,
        Offences: pallet_offences,
        Historical: pallet_session_historical,
        Session: pallet_session,
        Grandpa: pallet_grandpa,
        ImOnline: pallet_im_online,
        AuthorityDiscovery: pallet_authority_discovery,

        // Election pallet. Only works with staking
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase,
        // Provides a semi-sorted list of nominators for staking.
        BagsList: pallet_bags_list,

        // Governance stuff. uncallable initially
        Sudo: pallet_sudo,
        Treasury: pallet_treasury,

        // Cunning utilities. Usable initially.
        Utility: pallet_utility,

        Multisig: pallet_multisig,
        Preimage: pallet_preimage,

        // Ternoa pallets.  Start indices at 100 to leave room.
        Nfts: ternoa_nfts = 100,
        AssociatedAccounts: ternoa_associated_accounts = 101,
        // Capsules: ternoa_capsules = 102,
        // Marketplace: ternoa_marketplace = 103,
        // Auctions: ternoa_auctions = 104,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
///
/// When you change this, you **MUST** modify [`sign`] in `bin/node/testing/src/keyring.rs`!
///
/// [`sign`]: <../../testing/src/keyring.rs.html>
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: BABE_GENESIS_EPOCH_CONFIG.c,
                genesis_authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot_number: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }

        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
            let weight = Executive::try_runtime_upgrade()?;
            Ok((weight, RuntimeBlockWeights::get().max_block))
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;

            // Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
            // issues. To get around that, we separated the Session benchmarks into its own crate,
            // which is why we need these two lines below.
            use frame_system_benchmarking::Pallet as SystemBench;
            use frame_benchmarking::baseline::Pallet as Baseline;


            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();
            return (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};
            //use pallet_session_benchmarking::Module as SessionBench;
            //use pallet_offences_benchmarking::Module as OffencesBench;
            use frame_system_benchmarking::Pallet as SystemBench;
            use frame_benchmarking::baseline::Pallet as Baseline;

            // those two depends on pallets we do not use and pause compile time issues
            //impl pallet_session_benchmarking::Config for Runtime {}
            //impl pallet_offences_benchmarking::Config for Runtime {}
            impl frame_system_benchmarking::Config for Runtime {}
            impl frame_benchmarking::baseline::Config for Runtime {}

            // We took this from the substrate examples as the configurations are pretty close.
            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
                // Treasury Account
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmarks!(params, batches);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        // Ternoa
        // NOTE: Make sure to prefix these with `runtime_common::` so
        // the that path resolves correctly in the generated file.
        [ternoa_nfts, Nfts]
        [ternoa_associated_accounts, AssociatedAccounts]
        // Substrate
        [pallet_babe, Babe]
        [pallet_timestamp, Timestamp]
        // [pallet_indices, Indices]
        [pallet_balances, Balances]
        // [pallet_transaction_payment, TransactionPayment]
        // [pallet_authorship, Authorship]
        [pallet_staking, Staking]
        // [pallet_offences, Offences]
        // [pallet_session_historical, Historical]
        // [pallet_session, Session]
        [pallet_grandpa, Grandpa]
        [pallet_im_online, ImOnline]
        // [pallet_authority_discovery, AuthorityDiscovery]
        // [pallet_sudo, Sudo]
        [pallet_treasury, Treasury]
        [pallet_utility, Utility]
        [pallet_election_provider_multi_phase, ElectionProviderMultiPhase]
        [pallet_bags_list, BagsList]
        // [pallet_multisig, Multisig]
        // [pallet_preimage, Preimage]
        [frame_benchmarking::baseline, Baseline::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
    );
}
