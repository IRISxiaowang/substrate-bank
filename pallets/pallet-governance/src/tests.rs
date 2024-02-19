#![cfg(test)]

use crate::{mock::*, *};

use frame_support::{assert_noop, assert_ok};

#[test]
fn can_initiate_proposal() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Verify the authorities are set.
			assert_eq!(CurrentAuthorities::<Runtime>::get(), (11..21).collect::<BTreeSet<_>>());

			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let authority_member: AccountId = 11;
			let first_block = System::block_number();
			let first_proposal = 1u32;

			// Test the function initiate_proposal works.
			assert_ok!(Governance::initiate_proposal(
				RuntimeOrigin::signed(authority_member),
				call.clone()
			));

			// Verify
			assert_eq!(Proposals::<Runtime>::get(first_proposal), Some(call.encode()));

			assert_eq!(
				Expiry::<Runtime>::get(first_block + EXPIRY_PERIOD),
				BTreeSet::from_iter(vec![first_proposal])
			);

			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes {
					yays: BTreeSet::from_iter(vec![authority_member]),
					nays: Default::default()
				}
			);

			System::assert_last_event(RuntimeEvent::Governance(
				Event::<Runtime>::ProposalRegistered { who: authority_member, call: *call },
			));
		});
}

#[test]
fn cannot_initiate_proposal_with_invaild_authority() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Verify the authorities are set.
			assert_eq!(CurrentAuthorities::<Runtime>::get(), (11..21).collect::<BTreeSet<_>>());

			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let invaild_authority_member: AccountId = 10;

			// Test the function initiate_proposal works.
			assert_noop!(
				Governance::initiate_proposal(
					RuntimeOrigin::signed(invaild_authority_member),
					call.clone()
				),
				Error::<Runtime>::Unauthorized
			);
		});
}

#[test]
fn can_vote() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data.
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let authority_member_1: AccountId = 13;
			let authority_member_2: AccountId = 12;
			let authority_member_3: AccountId = 11;

			let first_proposal = 1u32;
			Proposals::<Runtime>::set(first_proposal, Some(call.encode()));

			// There are 7 persons voted yes, to test when the 8th person vote, it not resolve yet,
			// but after the 9th person voted, it will be reserved.
			Votes::<Runtime>::set(
				first_proposal,
				CastedVotes { yays: (14..21).collect::<BTreeSet<_>>(), nays: Default::default() },
			);

			// Test the function vote works.
			assert_ok!(Governance::vote(
				RuntimeOrigin::signed(authority_member_1),
				first_proposal,
				true
			));

			// Verify
			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes { yays: (13..21).collect::<BTreeSet<_>>(), nays: Default::default() }
			);
			assert_eq!(ProposalsToResolve::<Runtime>::get(), Default::default());

			System::assert_last_event(RuntimeEvent::Governance(Event::<Runtime>::VoteCasted {
				who: authority_member_1,
				proposal: first_proposal,
				approve: true,
			}));

			// Vote again.
			assert_ok!(Governance::vote(
				RuntimeOrigin::signed(authority_member_2),
				first_proposal,
				true
			));

			// Verify
			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes { yays: (12..21).collect::<BTreeSet<_>>(), nays: Default::default() }
			);
			assert_eq!(
				ProposalsToResolve::<Runtime>::get(),
				BTreeSet::from_iter(vec![(first_proposal, true)])
			);

			System::assert_last_event(RuntimeEvent::Governance(Event::<Runtime>::VoteCasted {
				who: authority_member_2,
				proposal: first_proposal,
				approve: true,
			}));

			// Vote again with reject.
			assert_ok!(Governance::vote(
				RuntimeOrigin::signed(authority_member_3),
				first_proposal,
				false
			));

			// Verify
			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes {
					yays: (12..21).collect::<BTreeSet<_>>(),
					nays: BTreeSet::from_iter(vec![11])
				}
			);
			assert_eq!(
				ProposalsToResolve::<Runtime>::get(),
				BTreeSet::from_iter(vec![(first_proposal, true)])
			);

			System::assert_last_event(RuntimeEvent::Governance(Event::<Runtime>::VoteCasted {
				who: authority_member_3,
				proposal: first_proposal,
				approve: false,
			}));
		});
}

#[test]
fn cannot_vote_with_invaild_authority() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data.
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let invaild_authority_member: AccountId = 10;
			let first_proposal = 1u32;
			Proposals::<Runtime>::set(first_proposal, Some(call.encode()));

			// Ensures that an unauthorized member cannot vote on a proposal.
			assert_noop!(
				Governance::vote(
					RuntimeOrigin::signed(invaild_authority_member),
					first_proposal,
					true
				),
				Error::<Runtime>::Unauthorized
			);
		});
}

#[test]
fn cannot_vote_with_invaild_proposal() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data.
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let authority_member: AccountId = 11;
			let first_proposal = 1u32;
			let invaild_proposal = 2u32;
			Proposals::<Runtime>::set(first_proposal, Some(call.encode()));

			// Ensures that an unauthorized member cannot vote on a proposal.
			assert_noop!(
				Governance::vote(RuntimeOrigin::signed(authority_member), invaild_proposal, true),
				Error::<Runtime>::InvalidProposalId
			);
		});
}

#[test]
fn cannot_vote_twice() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data.
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let has_voted_authority_member: AccountId = 11;
			let first_proposal = 1u32;
			Proposals::<Runtime>::set(first_proposal, Some(call.encode()));

			Votes::<Runtime>::set(
				first_proposal,
				CastedVotes { yays: (11..15).collect::<BTreeSet<_>>(), nays: Default::default() },
			);

			// Ensures that an unauthorized member cannot vote on a proposal.
			assert_noop!(
				Governance::vote(
					RuntimeOrigin::signed(has_voted_authority_member),
					first_proposal,
					true
				),
				Error::<Runtime>::AlreadyVoted
			);
		});
}

#[test]
fn can_force_rotate_authorities() {
	default_test_ext().execute_with(|| {
		// Force rotate authorities with some duplicated number, which tests duplicated number
		// would not affect results.
		assert_ok!(Governance::force_rotate_authorities(
			RuntimeOrigin::root(),
			(31..41).chain(31..35).collect::<Vec<_>>()
		));

		// Verify the authorities are set.
		assert_eq!(CurrentAuthorities::<Runtime>::get(), (31..41).collect::<BTreeSet<_>>());

		System::assert_last_event(RuntimeEvent::Governance(Event::<Runtime>::AuthorityRotated {
			new_council: (31..41).collect::<BTreeSet<_>>(),
		}));
	});
}

#[test]
fn can_end_to_end_council_rotate_authorities() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let authority_member: AccountId = 11;
			let first_proposal = 1u32;

			// Propose
			assert_ok!(Governance::initiate_proposal(
				RuntimeOrigin::signed(authority_member),
				call.clone()
			));

			// Vote
			for i in 12..21 {
				assert_ok!(Governance::vote(RuntimeOrigin::signed(i), first_proposal, true));
			}

			// dispatch
			Governance::on_finalize(System::block_number());

			// Verify the authorities are set.
			assert_eq!(CurrentAuthorities::<Runtime>::get(), (21..31).collect::<BTreeSet<_>>());

			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes { yays: Default::default(), nays: Default::default() }
			);
			assert_eq!(ProposalsToResolve::<Runtime>::get(), Default::default());
			assert_eq!(Proposals::<Runtime>::get(first_proposal), None);

			System::assert_has_event(RuntimeEvent::Governance(
				Event::<Runtime>::AuthorityRotated {
					new_council: (21..31).collect::<BTreeSet<_>>(),
				},
			));

			let last_event = System::events().last().unwrap().event.clone();
			matches!(last_event, RuntimeEvent::Governance(Event::<Runtime>::ProposalPassed {
			id,
			call,
			result: Ok(..),
		}) if id == first_proposal && call == call );
		});
}

#[test]
fn can_expired_council_rotate_authorities() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let authority_member: AccountId = 11;
			let first_proposal = 1u32;

			// Propose
			assert_ok!(Governance::initiate_proposal(
				RuntimeOrigin::signed(authority_member),
				call.clone()
			));

			// The number of votes is not enough to resolve the proposal.
			for i in 12..15 {
				assert_ok!(Governance::vote(RuntimeOrigin::signed(i), first_proposal, true));
			}

			// Expire the proposal.
			Governance::on_finalize(System::block_number() + EXPIRY_PERIOD);

			// Verify the authorities are original.
			assert_eq!(CurrentAuthorities::<Runtime>::get(), (11..21).collect::<BTreeSet<_>>());

			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes { yays: Default::default(), nays: Default::default() }
			);
			assert_eq!(ProposalsToResolve::<Runtime>::get(), Default::default());
			assert_eq!(Proposals::<Runtime>::get(first_proposal), None);
			assert_eq!(
				Expiry::<Runtime>::get(System::block_number() + EXPIRY_PERIOD),
				Default::default()
			);
		});
}

#[test]
fn can_reject_council_rotate_authorities() {
	MockGenesisConfig::with_authorities((11..21).collect::<Vec<_>>())
		.build()
		.execute_with(|| {
			// Setup data
			let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
				new_members: (21..31).collect::<Vec<_>>(),
			}));
			let authority_member: AccountId = 11;
			let first_proposal = 1u32;

			// Propose
			assert_ok!(Governance::initiate_proposal(
				RuntimeOrigin::signed(authority_member),
				call.clone()
			));

			// Vote 2 tickets rejected.
			for i in 12..14 {
				assert_ok!(Governance::vote(RuntimeOrigin::signed(i), first_proposal, false));
			}

			// dispatch
			Governance::on_finalize(System::block_number());

			// Verify the authorities are set.
			assert_eq!(CurrentAuthorities::<Runtime>::get(), (11..21).collect::<BTreeSet<_>>());

			assert_eq!(
				Votes::<Runtime>::get(first_proposal),
				CastedVotes { yays: Default::default(), nays: Default::default() }
			);
			assert_eq!(ProposalsToResolve::<Runtime>::get(), Default::default());
			assert_eq!(Proposals::<Runtime>::get(first_proposal), None);

			System::assert_has_event(RuntimeEvent::Governance(
				Event::<Runtime>::ProposalRejected {
					id: first_proposal,
					reason: RejectReason::ByVoting,
				},
			));
		});
}

#[test]
fn can_resolve_after_proposal() {
	MockGenesisConfig::with_authorities(vec![11]).build().execute_with(|| {
		let call = Box::new(RuntimeCall::Governance(crate::Call::council_rotate_authorities {
			new_members: (21..31).collect::<Vec<_>>(),
		}));
		let authority_member: AccountId = 11;
		let first_proposal = 1u32;

		// Test the function initiate_proposal works.
		assert_ok!(Governance::initiate_proposal(
			RuntimeOrigin::signed(authority_member),
			call.clone()
		));

		// Dispatch
		Governance::on_finalize(System::block_number());

		// Verify the authorities are set.
		assert_eq!(CurrentAuthorities::<Runtime>::get(), (21..31).collect::<BTreeSet<_>>());

		assert_eq!(Proposals::<Runtime>::get(first_proposal), None);

		assert_eq!(
			Votes::<Runtime>::get(first_proposal),
			CastedVotes { yays: Default::default(), nays: Default::default() }
		);

		let last_event = System::events().last().unwrap().event.clone();
		matches!(last_event, RuntimeEvent::Governance(Event::<Runtime>::ProposalPassed {
			id,
			call,
			result: Ok(..),
		}) if id == first_proposal && call == call );
	});
}
