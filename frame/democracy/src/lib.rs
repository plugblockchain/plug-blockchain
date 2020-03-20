#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{Hashable, assert_ok, assert_noop, parameter_types, weights::Weight};
	use frame_system::{self as system, EventRecord, Phase};
	use hex_literal::hex;
	use sp_core::H256;
	use sp_runtime::{
		Perbill, traits::{BlakeTwo256, IdentityLookup, Block as BlockT}, testing::Header,
		BuildStorage,
	};
	use crate as collective;

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
		pub const MotionDuration: u64 = 3;
	}
	impl frame_system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
		type DelegatedDispatchVerifer = ();
		type Doughnut = ();
	}
	impl Trait<Instance1> for Test {
		type Origin = Origin;
		type Proposal = Call;
		type Event = Event;
		type MotionDuration = MotionDuration;
	}
	impl Trait for Test {
		type Origin = Origin;
		type Proposal = Call;
		type Event = Event;
		type MotionDuration = MotionDuration;
	}

	pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
	pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic
		{
			System: system::{Module, Call, Event<T>},
			Collective: collective::<Instance1>::{Module, Call, Event<T>, Origin<T>, Config<T>},
			DefaultCollective: collective::{Module, Call, Event<T>, Origin<T>, Config<T>},
		}
	);

	fn make_ext() -> sp_io::TestExternalities {
		GenesisConfig {
			collective_Instance1: Some(collective::GenesisConfig {
				members: vec![1, 2, 3],
				phantom: Default::default(),
			}),
			collective: None,
		}.build_storage().unwrap().into()
	}

	#[test]
	fn motions_basic_environment_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			assert_eq!(Collective::members(), vec![1, 2, 3]);
			assert_eq!(Collective::proposals(), Vec::<H256>::new());
		});
	}

	fn make_proposal(value: u64) -> Call {
		Call::System(frame_system::Call::remark(value.encode()))
	}

	#[test]
	fn close_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash = BlakeTwo256::hash_of(&proposal);

			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, true));

			System::set_block_number(3);
			assert_noop!(
				Collective::close(Origin::signed(4), hash.clone(), 0),
				Error::<Test, Instance1>::TooEarly
			);

			System::set_block_number(4);
			assert_ok!(Collective::close(Origin::signed(4), hash.clone(), 0));

			let record = |event| EventRecord { phase: Phase::Finalization, event, topics: vec![] };
			assert_eq!(System::events(), vec![
				record(Event::collective_Instance1(RawEvent::Proposed(1, 0, hash.clone(), 3))),
				record(Event::collective_Instance1(RawEvent::Voted(2, hash.clone(), true, 2, 0))),
				record(Event::collective_Instance1(RawEvent::Closed(hash.clone(), 2, 1))),
				record(Event::collective_Instance1(RawEvent::Disapproved(hash.clone())))
			]);
		});
	}

	#[test]
	fn close_with_prime_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash = BlakeTwo256::hash_of(&proposal);
			assert_ok!(Collective::set_members(Origin::ROOT, vec![1, 2, 3], Some(3)));

			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, true));

			System::set_block_number(4);
			assert_ok!(Collective::close(Origin::signed(4), hash.clone(), 0));

			let record = |event| EventRecord { phase: Phase::Finalization, event, topics: vec![] };
			assert_eq!(System::events(), vec![
				record(Event::collective_Instance1(RawEvent::Proposed(1, 0, hash.clone(), 3))),
				record(Event::collective_Instance1(RawEvent::Voted(2, hash.clone(), true, 2, 0))),
				record(Event::collective_Instance1(RawEvent::Closed(hash.clone(), 2, 1))),
				record(Event::collective_Instance1(RawEvent::Disapproved(hash.clone())))
			]);
		});
	}

	#[test]
	fn close_with_voting_prime_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash = BlakeTwo256::hash_of(&proposal);
			assert_ok!(Collective::set_members(Origin::ROOT, vec![1, 2, 3], Some(1)));

			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, true));

			System::set_block_number(4);
			assert_ok!(Collective::close(Origin::signed(4), hash.clone(), 0));

			let record = |event| EventRecord { phase: Phase::Finalization, event, topics: vec![] };
			assert_eq!(System::events(), vec![
				record(Event::collective_Instance1(RawEvent::Proposed(1, 0, hash.clone(), 3))),
				record(Event::collective_Instance1(RawEvent::Voted(2, hash.clone(), true, 2, 0))),
				record(Event::collective_Instance1(RawEvent::Closed(hash.clone(), 3, 0))),
				record(Event::collective_Instance1(RawEvent::Approved(hash.clone()))),
				record(Event::collective_Instance1(RawEvent::Executed(hash.clone(), false)))
			]);
		});
	}

	#[test]
	fn removal_of_old_voters_votes_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash = BlakeTwo256::hash_of(&proposal);
			let end = 4;
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, true));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 3, ayes: vec![1, 2], nays: vec![], end })
			);
			Collective::change_members_sorted(&[4], &[1], &[2, 3, 4]);
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 3, ayes: vec![2], nays: vec![], end })
			);

			let proposal = make_proposal(69);
			let hash = BlakeTwo256::hash_of(&proposal);
			assert_ok!(Collective::propose(Origin::signed(2), 2, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(3), hash.clone(), 1, false));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 1, threshold: 2, ayes: vec![2], nays: vec![3], end })
			);
			Collective::change_members_sorted(&[], &[3], &[2, 4]);
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 1, threshold: 2, ayes: vec![2], nays: vec![], end })
			);
		});
	}

	#[test]
	fn removal_of_old_voters_votes_works_with_set_members() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash = BlakeTwo256::hash_of(&proposal);
			let end = 4;
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, true));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 3, ayes: vec![1, 2], nays: vec![], end })
			);
			assert_ok!(Collective::set_members(Origin::ROOT, vec![2, 3, 4], None));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 3, ayes: vec![2], nays: vec![], end })
			);

			let proposal = make_proposal(69);
			let hash = BlakeTwo256::hash_of(&proposal);
			assert_ok!(Collective::propose(Origin::signed(2), 2, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(3), hash.clone(), 1, false));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 1, threshold: 2, ayes: vec![2], nays: vec![3], end })
			);
			assert_ok!(Collective::set_members(Origin::ROOT, vec![2, 4], None));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 1, threshold: 2, ayes: vec![2], nays: vec![], end })
			);
		});
	}

	#[test]
	fn propose_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash = proposal.blake2_256().into();
			let end = 4;
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_eq!(Collective::proposals(), vec![hash]);
			assert_eq!(Collective::proposal_of(&hash), Some(proposal));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 3, ayes: vec![1], nays: vec![], end })
			);

			assert_eq!(System::events(), vec![
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Proposed(
						1,
						0,
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						3,
					)),
					topics: vec![],
				}
			]);
		});
	}

	#[test]
	fn motions_ignoring_non_collective_proposals_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			assert_noop!(
				Collective::propose(Origin::signed(42), 3, Box::new(proposal.clone())),
				Error::<Test, Instance1>::NotMember
			);
		});
	}

	#[test]
	fn motions_ignoring_non_collective_votes_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash: H256 = proposal.blake2_256().into();
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_noop!(
				Collective::vote(Origin::signed(42), hash.clone(), 0, true),
				Error::<Test, Instance1>::NotMember,
			);
		});
	}

	#[test]
	fn motions_ignoring_bad_index_collective_vote_works() {
		make_ext().execute_with(|| {
			System::set_block_number(3);
			let proposal = make_proposal(42);
			let hash: H256 = proposal.blake2_256().into();
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_noop!(
				Collective::vote(Origin::signed(2), hash.clone(), 1, true),
				Error::<Test, Instance1>::WrongIndex,
			);
		});
	}

	#[test]
	fn motions_revoting_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash: H256 = proposal.blake2_256().into();
			let end = 4;
			assert_ok!(Collective::propose(Origin::signed(1), 2, Box::new(proposal.clone())));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 2, ayes: vec![1], nays: vec![], end })
			);
			assert_noop!(
				Collective::vote(Origin::signed(1), hash.clone(), 0, true),
				Error::<Test, Instance1>::DuplicateVote,
			);
			assert_ok!(Collective::vote(Origin::signed(1), hash.clone(), 0, false));
			assert_eq!(
				Collective::voting(&hash),
				Some(Votes { index: 0, threshold: 2, ayes: vec![], nays: vec![1], end })
			);
			assert_noop!(
				Collective::vote(Origin::signed(1), hash.clone(), 0, false),
				Error::<Test, Instance1>::DuplicateVote,
			);

			assert_eq!(System::events(), vec![
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Proposed(
						1,
						0,
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						2,
					)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Voted(
						1,
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						false,
						0,
						1,
					)),
					topics: vec![],
				}
			]);
		});
	}

	#[test]
	fn motions_reproposing_disapproved_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash: H256 = proposal.blake2_256().into();
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, false));
			assert_eq!(Collective::proposals(), vec![]);
			assert_ok!(Collective::propose(Origin::signed(1), 2, Box::new(proposal.clone())));
			assert_eq!(Collective::proposals(), vec![hash]);
		});
	}

	#[test]
	fn motions_disapproval_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash: H256 = proposal.blake2_256().into();
			assert_ok!(Collective::propose(Origin::signed(1), 3, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, false));

			assert_eq!(System::events(), vec![
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(
						RawEvent::Proposed(
							1,
							0,
							hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
							3,
						)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Voted(
						2,
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						false,
						1,
						1,
					)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Disapproved(
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
					)),
					topics: vec![],
				}
			]);
		});
	}

	#[test]
	fn motions_approval_works() {
		make_ext().execute_with(|| {
			System::set_block_number(1);
			let proposal = make_proposal(42);
			let hash: H256 = proposal.blake2_256().into();
			assert_ok!(Collective::propose(Origin::signed(1), 2, Box::new(proposal.clone())));
			assert_ok!(Collective::vote(Origin::signed(2), hash.clone(), 0, true));

			assert_eq!(System::events(), vec![
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Proposed(
						1,
						0,
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						2,
					)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Voted(
						2,
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						true,
						2,
						0,
					)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Approved(
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
					)),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Finalization,
					event: Event::collective_Instance1(RawEvent::Executed(
						hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
						false,
					)),
					topics: vec![],
				}
			]);
		});
	}
}