#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use pallet_auction::AuctionDataFor;
use pallet_grandpa::AuthorityId as GrandpaId;
use pallet_nft::NftData;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, NumberFor, One},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Percent,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::genesis_builder_helper::{build_config, create_default_config};
// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness,
		StorageInfo,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		IdentityFee, Weight,
	},
	StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

use primitives::{
	AccountId, AuctionId, Balance, BlockNumber, Hash, LockId, NftId, Nonce, PendingNftPods,
	RpcNftData, Signature, DAY, DOLLAR, HOUR, SLOT_DURATION, YEAR,
};

pub mod runtime_api;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("xy-chain"),
	impl_name: create_runtime_str!("xy-chain"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::with_sensible_defaults(
			Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;

	#[cfg(feature = "experimental")]
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<DOLLAR>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type MaxHolds = ();
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub MinimumAmount: Balance = 2 * DOLLAR;
}

impl pallet_bank::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_bank::weights::SubstrateWeight<Runtime>;
	type Balance = Balance;
	type RoleManager = Roles;
	type EnsureGovernance = pallet_governance::EnsureGovernance;
	type ExistentialDeposit = ConstU128<DOLLAR>;
	type MinimumAmount = MinimumAmount;
	type RedeemPeriod = ConstU32<{ 5 * DAY }>;
	type StakePeriod = ConstU32<{ 2 * DAY }>;
	type InterestPayoutPeriod = ConstU32<DAY>;
	type TotalBlocksPerYear = ConstU32<YEAR>;
}

/// Configure the pallet-template in pallets/template.
impl pallet_roles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_roles::weights::SubstrateWeight<Runtime>;
	type EnsureGovernance = pallet_governance::EnsureGovernance;
}

parameter_types! {
	pub const LotteryPayoutPeriod: BlockNumber = DAY as BlockNumber;
	pub PrizePoolAccount: AccountId = AccountId::from([0xFF; 32]);
	pub TaxRate: Percent = Percent::from_percent(5);
}

impl pallet_lottery::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_lottery::weights::SubstrateWeight<Runtime>;
	type Balance = Balance;
	type RoleManager = Roles;
	type EnsureGovernance = pallet_governance::EnsureGovernance;
	type Bank = Bank;
	type Randomness = Random;
	type LotteryPayoutPeriod = LotteryPayoutPeriod;
	type PrizePoolAccount = PrizePoolAccount;
	type TaxRate = TaxRate;
}

parameter_types! {
	pub const ExpiryPeriod: BlockNumber = DAY as BlockNumber;
	pub const MajorityThreshold: Percent = Percent::from_percent(80);
}

impl pallet_governance::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type ExpiryPeriod = ExpiryPeriod;
	type MajorityThreshold = MajorityThreshold;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type EnsureGovernance = pallet_governance::EnsureGovernance;
}

impl pallet_nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RoleManager = Roles;
	type Balance = Balance;
	type Bank = Bank;
	type AuctionManager = Auction;
	type EnsureGovernance = pallet_governance::EnsureGovernance;
	type MaxSize = ConstU32<1_048_576>; // 1MB
	type PodFee = ConstU128<DOLLAR>;
	type NftLockedPeriod = ConstU32<DAY>;
}

parameter_types! {
	pub BidsPoolAccount: AccountId = AccountId::from([0xEE; 32]);
	pub AuctionSuccessFeePercentage: Percent = Percent::from_percent(10);
	pub const AuctionLength: BlockNumber = DAY as BlockNumber;
	pub const ExtendedLength: BlockNumber = HOUR as BlockNumber;
}

impl pallet_auction::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RoleManager = Roles;
	type Balance = Balance;
	type Bank = Bank;
	type NftManager = Nft;
	type BidsPoolAccount = BidsPoolAccount;
	type AuctionSuccessFeePercentage = AuctionSuccessFeePercentage;
	type AuctionStartFee = ConstU128<DOLLAR>;
	type MinimumIncrease = ConstU128<DOLLAR>;
	type AuctionLength = AuctionLength;
	type ExtendedLength = ExtendedLength;
}
// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		Sudo: pallet_sudo,
		Random: pallet_insecure_randomness_collective_flip,
		// Include the custom logic from the pallet-template in the runtime.
		Bank: pallet_bank,
		Roles: pallet_roles,
		Lottery: pallet_lottery,
		Governance: pallet_governance,
		Nft: pallet_nft,
		Auction: pallet_auction,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_sudo, Sudo]
		[pallet_bank, Bank]
		[pallet_roles, Roles]
		[pallet_lottery, Lottery]
		[pallet_governance, Governance]
		[pallet_nft, Nft]
		[pallet_auction, Auction]
	);
}

impl_runtime_apis! {
	impl runtime_api::CustomRuntimeApi<Block> for Runtime {
		/// Returns account Data for a user
		fn account_data(who: AccountId) -> pallet_bank::AccountData<Balance> {
			Bank::accounts(who)
		}
		/// Calculate and returns the actual interest return per annum.
		fn interest_pa(who: AccountId) -> Balance {
			Bank::interest_pa(who)
		}
		/// Returns when a locked fund is released.
		fn fund_unlock_at(who: AccountId, lock_id: LockId) -> BlockNumber {
			Bank::fund_unlock_at(who, lock_id)
		}

		/// Returns certain user's related Nft in POD info.
		fn pending_pods(who: AccountId) -> PendingNftPods {
			PendingNftPods {
				delivering: pallet_nft::PendingPodNfts::<Runtime>::iter()
					.filter_map(|(pod_id, pod_info)| {
						(who ==
							pallet_nft::Owners::<Runtime>::get(pod_info.nft_id)
								.expect("Nft in POD must have an owner."))
						.then_some(RpcNftData {
							pod_id,
							sender: who.clone(),
							nft_id: pod_info.nft_id,
							nft_name: pallet_nft::Nfts::<Runtime>::get(pod_info.nft_id)
								.expect("Nft in POD must have an owner.")
								.file_name,
							expiry_block: pod_info.expiry_block,
							price: pod_info.price,
						})
					})
					.collect(),
				receiving: pallet_nft::PendingPodNfts::<Runtime>::iter()
					.filter_map(|(pod_id, pod_info)| {
						(pod_info.to_user == who).then_some(RpcNftData {
							pod_id,
							sender: pallet_nft::Owners::<Runtime>::get(pod_info.nft_id)
								.expect("Nft in POD must have an owner."),
							nft_id: pod_info.nft_id,
							nft_name: pallet_nft::Nfts::<Runtime>::get(pod_info.nft_id)
								.expect("Nft in POD must have an owner.")
								.file_name,
							expiry_block: pod_info.expiry_block,
							price: pod_info.price,
						})
					})
					.collect(),
			}
		}

		/// Returns all the current auctions without auction id,
		///  or return a specific auction info with an auction id.
		fn current_auctions(auction_id: Option<AuctionId>) -> Vec<(AuctionId, AuctionDataFor<Runtime>)>{
			match auction_id{
				Some(id) => pallet_auction::Auctions::<Runtime>::get(id)
					.map(|auction|vec![(id, auction)])
					.unwrap_or_default(),
				None => pallet_auction::Auctions::<Runtime>::iter().collect(),
			}
		}

		/// Return a specific NFT data with a NFT id.
		fn nft_data(nft_id: NftId) -> Option<NftData>{
			pallet_nft::Nfts::<Runtime>::get(nft_id)
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
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

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}
}
