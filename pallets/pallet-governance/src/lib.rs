//! # Governance Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::GetDispatchInfo,
	pallet_prelude::*,
	storage::with_transaction,
	traits::{BuildGenesisConfig, UnfilteredDispatchable},
};
use frame_system::pallet_prelude::*;
use primitives::ProposalId;
use sp_runtime::{traits::Saturating, DispatchResult, Percent, TransactionOutcome};
use sp_std::{collections::btree_set::BTreeSet, prelude::*, vec::Vec};

// mod mock;
// mod tests;

pub mod weights;
pub use weights::*;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum RejectReason {
	Expired,
	ByVoting,
}

/// Stores casted votes.
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct CastedVotes<AccountId> {
	pub yays: BTreeSet<AccountId>,
	pub nays: BTreeSet<AccountId>,
}

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;
	use sp_runtime::traits::BlockNumberProvider;

	type EncodedCall = Vec<u8>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		/// The outer Origin needs to be compatible with this pallet's Origin
		type RuntimeOrigin: From<RawOrigin>
			+ From<frame_system::RawOrigin<<Self as frame_system::Config>::AccountId>>;

		/// The overarching call type.
		type RuntimeCall: Member
			+ Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>
			+ From<frame_system::Call<Self>>
			+ From<Call<Self>>
			+ GetDispatchInfo;

		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

		type EnsureGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		#[pallet::constant]
		type ExpiryPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type MajorityThreshold: Get<Percent>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The member is not inside the council.
		InvalidAuthority,
		/// The proposal id does not exist.
		InvalidProposalId,
		/// The user has voted.
		AlreadyVoted,
		/// Error wouldn't like to happen.
		ImpossibleReach,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Registered the person who made the proposal.
		ProposalRegistered { who: T::AccountId, call: <T as Config>::RuntimeCall },

		/// Registered the vote result.
		VoteCasted { who: T::AccountId, proposal: ProposalId, approve: bool },

		/// Rotated the council members.
		AuthorityRotated { new_council: BTreeSet<T::AccountId> },

		/// Registered the passed proposal.
		ProposalPassed {
			id: ProposalId,
			call: <T as Config>::RuntimeCall,
			result: DispatchResultWithPostInfo,
		},

		/// Registered the rejected proposal.
		ProposalRejected { id: ProposalId, reason: RejectReason },
	}

	/// Stores the next Proposal ID should be.
	#[pallet::storage]
	pub type NextProposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

	/// Stores the proposal id related the call.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, ProposalId, EncodedCall>;

	/// Stores the proposal IDs that will expire at a black.
	#[pallet::storage]
	#[pallet::getter(fn expiry)]
	pub type Expiry<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, BTreeSet<ProposalId>>;

	/// Stores the vote results of the proposal id.
	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageMap<_, Blake2_128Concat, ProposalId, CastedVotes<T::AccountId>>;

	/// Stores the current council members.
	#[pallet::storage]
	#[pallet::getter(fn authorities)]
	pub type CurrentAuthorities<T: Config> = StorageValue<_, BTreeSet<T::AccountId>, ValueQuery>;

	/// Stores the resolved proposals.
	#[pallet::storage]
	#[pallet::getter(fn resolve)]
	pub type ProposalsToResolve<T: Config> =
		StorageValue<_, BTreeSet<(ProposalId, bool)>, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		initial_authorities: Vec<T::AccountId>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// Set CurrentAuthorities storage with initial_authorities.
			Pallet::<T>::rotate_authorities(self.initial_authorities.clone());
		}
	}

	#[pallet::origin]
	pub type Origin = RawOrigin;

	/// The raw origin enum for this pallet.
	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub enum RawOrigin {
		GovernanceApproval,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			// Check the expiry
			if let Some(expired_proposals) = Expiry::<T>::get(block_number) {
				let _ = expired_proposals.iter().map(|proposal_id| {
					Proposals::<T>::remove(proposal_id);
					Votes::<T>::remove(proposal_id);
				});
			}

			// Make the function call
			ProposalsToResolve::<T>::take().into_iter().for_each(|(proposal, approved)| {
				if let Some(Ok(call)) = Proposals::<T>::take(proposal)
					.map(|encoded| <T as Config>::RuntimeCall::decode(&mut &(*encoded)))
				{
					if approved {
						Self::deposit_event(Event::<T>::ProposalPassed {
							id: proposal,
							call: call.clone(),
							result: Self::dispatch_governance_call(call),
						});
					} else {
						Self::deposit_event(Event::<T>::ProposalRejected {
							id: proposal,
							reason: RejectReason::ByVoting,
						});
					}
				}
				// Deleted the related data in Votes.
				Votes::<T>::remove(proposal);
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registered a proposal.
		///
		/// Require a member from council.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::initiate_proposal())]
		pub fn initiate_proposal(
			origin: OriginFor<T>,
			call: <T as Config>::RuntimeCall,
		) -> DispatchResult {
			let id = ensure_signed(origin)?;
			ensure!(CurrentAuthorities::<T>::get().contains(&id), Error::<T>::InvalidAuthority);
			let proposal_id = Self::next_proposal_id();
			let expired_block =
				T::BlockNumberProvider::current_block_number() + T::ExpiryPeriod::get();

			// add call to storage
			Proposals::<T>::insert(proposal_id, call.encode());

			// add proposal id to expiry.
			// Get the BTreeSet for the block number or create a new one if it doesn't exist
			let mut expiry_set = Expiry::<T>::get(expired_block).unwrap_or_default();

			// Insert the proposal id into the set
			expiry_set.insert(proposal_id);

			// Store the updated set back into storage
			Expiry::<T>::insert(expired_block, expiry_set);

			// add vote to Votes
			let mut set = BTreeSet::new();
			set.insert(id.clone());
			Votes::<T>::insert(proposal_id, CastedVotes { yays: set, nays: Default::default() });

			// Emit Event: ProposalRegistered
			Self::deposit_event(Event::<T>::ProposalRegistered { who: id, call });

			Ok(())
		}

		// Requires Root origin - sets the current authorities
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::vote())]
		pub fn vote(origin: OriginFor<T>, proposal: ProposalId, approve: bool) -> DispatchResult {
			// check if the vote can be casted
			// is member of the council
			let id = ensure_signed(origin)?;
			ensure!(CurrentAuthorities::<T>::get().contains(&id), Error::<T>::InvalidAuthority);

			// Proposal is valid
			ensure!(Proposals::<T>::get(proposal).is_some(), Error::<T>::InvalidProposalId);

			if let Some(mut casted) = Votes::<T>::get(proposal) {
				// has not voted
				ensure!(
					!casted.yays.contains(&id) && !casted.nays.contains(&id),
					Error::<T>::AlreadyVoted
				);

				// Cast the vote
				if approve {
					casted.yays.insert(id.clone());
				} else {
					casted.nays.insert(id.clone());
				}

				// check if the proposal is ready to be resolved.
				let count_yays = casted.yays.len();
				let count_nays = casted.nays.len();
				let count_council = CurrentAuthorities::<T>::get().len();
				let pass: Percent = Percent::from_rational(count_yays as u32, count_council as u32);
				let fail: Percent = Percent::from_rational(count_nays as u32, count_council as u32);

				if pass >= T::MajorityThreshold::get() {
					ProposalsToResolve::<T>::get().insert((proposal, true));
				}

				if fail >= Percent::one().saturating_sub(T::MajorityThreshold::get()) {
					ProposalsToResolve::<T>::get().insert((proposal, false));
				}
			} else {
				// return error, can not reach here.
			}

			// Emit Event: VoteCasted
			Self::deposit_event(Event::<T>::VoteCasted { who: id, proposal, approve });

			Ok(())
		}

		// Requires Root origin - sets the current authorities
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::force_rotate_authorities())]
		pub fn force_rotate_authorities(
			origin: OriginFor<T>,
			new_members: Vec<T::AccountId>,
		) -> DispatchResult {
			// ensure root
			ensure_root(origin)?;
			Self::rotate_authorities(new_members);
			Ok(())
		}

		// Requires Governance origin - sets the current authorities
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::council_rotate_authorities())]
		pub fn council_rotate_authorities(
			origin: OriginFor<T>,
			new_members: Vec<T::AccountId>,
		) -> DispatchResult {
			// ensure governance
			T::EnsureGovernance::ensure_origin(origin)?;

			Self::rotate_authorities(new_members);
			Ok(())
		}
	}

	/// Custom governance origin
	pub struct EnsureGovernance;

	/// Implementation for EnsureOrigin trait for custom EnsureGovernance struct.
	/// We use this to execute extrinsic by a governance origin.
	impl<OuterOrigin> EnsureOrigin<OuterOrigin> for EnsureGovernance
	where
		OuterOrigin: Into<Result<RawOrigin, OuterOrigin>> + From<RawOrigin>,
	{
		type Success = ();

		fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
			match o.into() {
				Ok(o) => match o {
					RawOrigin::GovernanceApproval => Ok(()),
				},
				Err(o) => Err(o),
			}
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<OuterOrigin, ()> {
			Ok(RawOrigin::GovernanceApproval.into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Dispatches a call from the governance origin, with transactional semantics, ie. if the
		/// call dispatch returns `Err`, rolls back any storage updates.
		fn dispatch_governance_call(
			call: <T as Config>::RuntimeCall,
		) -> DispatchResultWithPostInfo {
			with_transaction(move || {
				match call.dispatch_bypass_filter(RawOrigin::GovernanceApproval.into()) {
					r @ Ok(_) => TransactionOutcome::Commit(r),
					r @ Err(_) => TransactionOutcome::Rollback(r),
				}
			})
		}

		/// Increased proposal ID.
		fn next_proposal_id() -> ProposalId {
			NextProposalId::<T>::mutate(|id| {
				*id = id.saturating_add(1);
				*id
			})
		}

		fn rotate_authorities(authorities: Vec<T::AccountId>) {
			// Sets the new authority
			let unique_members: BTreeSet<_> = authorities.into_iter().collect();
			CurrentAuthorities::<T>::set(unique_members.clone());

			// Reset all expiry blocks
			if Expiry::<T>::iter().next().is_some() {
				let expired_block = T::BlockNumberProvider::current_block_number()
					.saturating_add(T::ExpiryPeriod::get());
				// Initialize an empty set to store all elements
				let mut all_elements: BTreeSet<ProposalId> = BTreeSet::new();

				// Iterate through the storage map and collect all proposal ids.
				for (_, expiry_set) in Expiry::<T>::iter() {
					// Merge the current expiry set with the all_elements set
					all_elements.extend(expiry_set.into_iter());
				}
				let _ = Expiry::<T>::clear(u32::MAX, None);
				Expiry::<T>::insert(expired_block, all_elements);
			}

			// Reset all votes for current proposals.
			for proposal_id in Votes::<T>::iter_keys() {
				if let Some(mut casted_votes) = Votes::<T>::get(proposal_id) {
					casted_votes.yays = BTreeSet::new();
					casted_votes.nays = BTreeSet::new();
					Votes::<T>::insert(proposal_id, casted_votes);
				}
			}

			// Emit event: AuthorityRotated
			Self::deposit_event(Event::<T>::AuthorityRotated { new_council: unique_members });
		}
	}
}
