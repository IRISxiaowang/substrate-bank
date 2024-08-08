#![cfg_attr(not(feature = "std"), no_std)]
//!
//! This crate contains basic primitive types used within the blockchain codebase.

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	MultiSignature, RuntimeDebug,
};
use sp_std::prelude::*;

pub mod constants;
pub use constants::*;

use serde::{Deserialize, Serialize};

/// Enum representing the different roles that a user can have.
#[derive(
	Encode,
	Decode,
	Copy,
	Clone,
	PartialEq,
	Eq,
	MaxEncodedLen,
	RuntimeDebug,
	TypeInfo,
	Serialize,
	Deserialize,
)]
pub enum Role {
	/// Represents a regular customer role.
	Customer,
	/// Represents a manager role with higher privileges.
	Manager,
	/// Represents an auditor role responsible for auditing.
	Auditor,
}

/// Enum representing the different state that an Nft can have.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum NftState {
	Free,
	POD(PodId),
	Auction(AuctionId),
}

/// Enum representing the different state that an Nft can have.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum Response {
	Accept,
	Reject,
}

pub use sp_runtime::traits::{BlakeTwo256, Hash as HashT};

/// The block number type used by Polkadot.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to type for a signature for a transaction on the relay chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Identifier for a chain. 32-bit should be plenty.
pub type ChainId = u32;

/// A hash of some data used by the relay chain.
pub type Hash = sp_core::H256;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Nonce = u32;

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10 m currency (`10^7`) at a
/// resolution to all for one second's worth of an annualised 50% reward be paid to a unit holder
/// (`10^11` unit denomination), or `10^18` total atomic units, to grow at 50%/year for 51 years
/// (`10^9` multiplier) for an eventual total of `10^27` units (27 significant decimal figures).
/// We round denomination to `10^12` (12 SDF), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// Lock Id
pub type LockId = u64;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Proposal Id
pub type ProposalId = u32;

/// Nft Id
pub type NftId = u32;

/// Pod Id
pub type PodId = u32;

/// Auction Id
pub type AuctionId = u32;

/// Contains information on a Nft that is currently in PoD (Paid On Delivery)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Serialize, Deserialize)]
pub struct RpcNftData {
	pub pod_id: PodId,
	pub sender: AccountId,
	pub nft_id: NftId,
	pub nft_name: Vec<u8>,
	pub expiry_block: BlockNumber,
	pub price: Balance,
}

/// Contains the details of NFTs on POD (Paid On Delivery)
/// that are either being sent or received by the customer.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Serialize, Deserialize)]
pub struct PendingNftPods {
	pub delivering: Vec<RpcNftData>, // Nft being delivered to someone else
	pub receiving: Vec<RpcNftData>,  // Nft that needs to be received
}
