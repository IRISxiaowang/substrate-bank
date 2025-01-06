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

mod mock;
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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

impl<AccountId: sp_std::cmp::Ord + Clone> CastedVotes<AccountId> {
	/// Checks if the account with the given ID has already voted on the proposal.
	pub fn has_voted(&self, id: &AccountId) -> bool {
		self.yays.contains(id) || self.nays.contains(id)
	}

	/// Casts a vote on the proposal for the specified account.
	///
	/// If the account has already voted, returns false.
	/// Otherwise, if the vote is cast successfully, returns true.
	pub fn cast_vote(&mut self, who: AccountId, approve: bool) -> bool {
		if self.has_voted(&who) {
			false
		} else {
			if approve {
				self.yays.insert(who.clone());
			} else {
				self.nays.insert(who);
			}
			true
		}
	}

	/// Checks if the proposal can be resolved based on the current votes and threshold.
	///
	/// Returns:
	/// - Some(true) if the proposal passes the threshold.
	/// - Some(false) if the proposal fails the threshold.
	/// - None if the proposal cannot be resolved yet.
	pub fn can_resolve(&self, total_authorities: u32, threshold: Percent) -> Option<bool> {
		let count_yays = self.yays.len();
		let count_nays = self.nays.len();

		let pass: Percent = Percent::from_rational(count_yays as u32, total_authorities);
		let fail: Percent = Percent::from_rational(count_nays as u32, total_authorities);

		if pass > threshold {
			Some(true)
		} else if fail >= Percent::one().saturating_sub(threshold) {
			Some(false)
		} else {
			None
		}
	}

	/// Remove all votes that are not in the `retain` set.
	pub fn cull_votes(&mut self, retain: BTreeSet<AccountId>) {
		// Retain the votes for members included in the new authorities
		self.yays.retain(|member| retain.contains(member));
		self.nays.retain(|member| retain.contains(member));
	}
}

impl<AccountId> Default for CastedVotes<AccountId> {
	fn default() -> Self {
		Self { yays: Default::default(), nays: Default::default() }
	}
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

		type EnsureGovernance: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		#[pallet::constant]
		type ExpiryPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type MajorityThreshold: Get<Percent>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The member is not inside the council.
		Unauthorized,
		/// The proposal id does not exist.
		InvalidProposalId,
		/// The user has voted.
		AlreadyVoted,
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
	#[pallet::getter(fn proposal_id)]
	pub type NextProposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

	/// Use the increased proposalId, stores the encoded extrinsic call.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, ProposalId, EncodedCall>;

	/// Stores the proposal IDs that are set to expire at a specific block number.
	#[pallet::storage]
	#[pallet::getter(fn expiry)]
	pub type Expiry<T: Config> =
		StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, BTreeSet<ProposalId>, ValueQuery>;

	/// Stores the vote results of the proposal id.
	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageMap<_, Blake2_128Concat, ProposalId, CastedVotes<T::AccountId>, ValueQuery>;

	/// Stores the current council members.
	#[pallet::storage]
	#[pallet::getter(fn authorities)]
	pub type CurrentAuthorities<T: Config> = StorageValue<_, BTreeSet<T::AccountId>, ValueQuery>;

	/// Proposals that can be resolved are stored here, until the next block's on_finalize,
	/// where the the proposal is accepted (call dispatched) or rejected.
	#[pallet::storage]
	#[pallet::getter(fn resolve)]
	pub type ProposalsToResolve<T: Config> =
		StorageValue<_, BTreeSet<(ProposalId, bool)>, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub initial_authorities: Vec<T::AccountId>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// Set CurrentAuthorities storage with initial_authorities.
			Pallet::<T>::do_rotate_authorities(self.initial_authorities.clone());
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
		/// It dispatches each governance call which stored in the ProposalsToResolve,
		/// and log the event passed or rejected.
		/// Clean up the Votes and Proposals after dispatching the governance call.
		/// Check the expiry proposals on the current block, log an event if it expired,
		/// and tidy up the related storage.
		fn on_finalize(block_number: BlockNumberFor<T>) {
			ProposalsToResolve::<T>::take().into_iter().for_each(|(proposal, approved)| {
				// Decode the proposal call if it exists.
				if let Some(Ok(call)) = Proposals::<T>::take(proposal)
					.map(|encoded| <T as Config>::RuntimeCall::decode(&mut &(*encoded)))
				{
					// if approved that dispatch the governance call and record the result.
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

			// Take the expiry and clean up the storage if any proposal expired and log an event.
			Expiry::<T>::take(block_number).into_iter().for_each(|proposal_id| {
				// The resolved proposals should be cleaned up already, in this case, only log an
				// event with the expired proposal.
				if Proposals::<T>::contains_key(proposal_id) {
					Self::deposit_event(Event::<T>::ProposalRejected {
						id: proposal_id,
						reason: RejectReason::Expired,
					});
				}

				Proposals::<T>::remove(proposal_id);
				Votes::<T>::remove(proposal_id);
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registered a proposal which needs voted by council members in limit time.
		/// Default the caller voted pass and log a register event.
		///
		/// Require a member from council.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::initiate_proposal())]
		pub fn initiate_proposal(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			let id = ensure_signed(origin)?;
			ensure!(CurrentAuthorities::<T>::get().contains(&id), Error::<T>::Unauthorized);
			let proposal_id = Self::next_proposal_id();
			let expired_block =
				frame_system::Pallet::<T>::current_block_number() + T::ExpiryPeriod::get();

			// add call to storage
			Proposals::<T>::insert(proposal_id, call.encode());

			// add proposal id to expiry.
			// Get the BTreeSet for the block number and add the proposal id into the set.
			Expiry::<T>::mutate(expired_block, |expiry_set| {
				// Insert the proposal id into the set
				expiry_set.insert(proposal_id);
			});

			// add vote to Votes.
			let _ = Self::do_vote(id.clone(), proposal_id, true);

			// Emit Event: ProposalRegistered
			Self::deposit_event(Event::<T>::ProposalRegistered { who: id, call: *call });

			Ok(())
		}

		/// Allows council members to cast their votes (approval or rejection) on a given proposal.
		/// It ensures that votes are cast by authorized members.
		/// Checks the validity of the proposal, handles voting, determines if the proposal is ready
		/// for resolution.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::vote())]
		pub fn vote(origin: OriginFor<T>, proposal: ProposalId, approve: bool) -> DispatchResult {
			// check if the vote can be casted
			// is member of the council
			let id = ensure_signed(origin)?;
			ensure!(CurrentAuthorities::<T>::get().contains(&id), Error::<T>::Unauthorized);

			// Proposal is valid
			ensure!(Proposals::<T>::contains_key(proposal), Error::<T>::InvalidProposalId);

			// Vote
			Self::do_vote(id.clone(), proposal, approve)
		}

		/// Requires Root origin - sets the current authorities.
		/// Clear old votes from old authorities and extend voting period for new authorities to
		/// vote.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::force_rotate_authorities())]
		pub fn force_rotate_authorities(
			origin: OriginFor<T>,
			new_members: Vec<T::AccountId>,
		) -> DispatchResult {
			// ensure root
			ensure_root(origin)?;
			Self::do_rotate_authorities(new_members);
			Ok(())
		}

		/// Requires Governance origin - sets the current authorities
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::council_rotate_authorities())]
		pub fn council_rotate_authorities(
			origin: OriginFor<T>,
			new_members: Vec<T::AccountId>,
		) -> DispatchResult {
			// ensure governance
			T::EnsureGovernance::ensure_origin(origin)?;

			Self::do_rotate_authorities(new_members);
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

		/// Returned the next proposal ID which is increased one each time.
		fn next_proposal_id() -> ProposalId {
			NextProposalId::<T>::mutate(|id| {
				*id = id.wrapping_add(1);
				*id
			})
		}

		/// Orchestrates the rotation of authorities by setting new authorities.
		/// Resetting expiry blocks, clearing votes for current proposals, and emitting an event to
		/// notify interested parties about the authority rotation and the new council members.
		fn do_rotate_authorities(authorities: Vec<T::AccountId>) {
			// Identify old members
			let old_members = CurrentAuthorities::<T>::take();

			// Sets the new authority
			let unique_members: BTreeSet<_> = authorities.into_iter().collect();
			CurrentAuthorities::<T>::set(unique_members.clone());

			// Identify members to retain votes for
			let members_to_retain: BTreeSet<_> =
				unique_members.intersection(&old_members).cloned().collect();

			if !members_to_retain.is_empty() {
				for proposal_id in Votes::<T>::iter_keys() {
					Votes::<T>::mutate(proposal_id, |casted| {
						// Retain the votes for members included in the new authorities
						casted.cull_votes(members_to_retain.clone());
					});
				}
			} else {
				// Reset all votes for current proposals.
				let _ = Votes::<T>::clear(u32::MAX, None);
			}
			// Reset all expiry blocks
			// Merge the current expiry set with the all_proposals set and clean up the Expiry
			// storage.
			let all_proposals =
				Expiry::<T>::drain().fold(BTreeSet::new(), |mut init, (_, mut expiry_set)| {
					init.append(&mut expiry_set);
					init
				});
			let expired_block = frame_system::Pallet::<T>::current_block_number()
				.saturating_add(T::ExpiryPeriod::get());

			Expiry::<T>::insert(expired_block, all_proposals);

			// Emit event: AuthorityRotated
			Self::deposit_event(Event::<T>::AuthorityRotated { new_council: unique_members });
		}

		fn do_vote(id: T::AccountId, proposal: ProposalId, approve: bool) -> DispatchResult {
			Votes::<T>::mutate(proposal, |casted| {
				// has not voted and cast the vote
				ensure!(casted.cast_vote(id.clone(), approve), Error::<T>::AlreadyVoted);

				// check if the proposal is ready to be resolved.
				if let Some(bool) = casted.can_resolve(
					CurrentAuthorities::<T>::get().len() as u32,
					T::MajorityThreshold::get(),
				) {
					ProposalsToResolve::<T>::mutate(|proposals| {
						proposals.insert((proposal, bool));
					});
				}

				// Emit Event: VoteCasted
				Self::deposit_event(Event::<T>::VoteCasted { who: id, proposal, approve });

				Ok(())
			})
		}
	}
}
