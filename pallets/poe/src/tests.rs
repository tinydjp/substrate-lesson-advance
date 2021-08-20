use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;
use crate::mock::Event;
use frame_system as system;

/// helper functions to retrieve events emited by extrinsics
/// reference: https://github.com/paritytech/substrate/blob/83942f58fc859ef5790351691e1ef665d79f0ead/frame/balances/src/tests.rs#L470-L473
fn last_event() -> Event {
	system::Pallet::<Test>::events().pop().expect("Event expected").event
}


#[test]
fn create_valid_claim_should_work() {
	new_test_ext().execute_with(|| {
		let claim = b"claim".to_vec();
		let sender = 1;
		// !IMPORTANT block number is 0 by default, events only emit when block number is greater than 0
		System::set_block_number(1);
		// postive case
		assert_ok!(PoeModule::create_claim(Origin::signed(sender), claim.clone()));
		// verify storage
		assert_eq!(
			Proofs::<Test>::get(&claim),
			(1, system::Pallet::<Test>::block_number())
		);
		// capture events and compare
		assert_eq!(
			last_event(),
			Event::PoeModule(crate::Event::ClaimCreated(sender, claim))
		)
	});
}

#[test]
fn create_invalid_claim_should_fail() {
	new_test_ext().execute_with(|| {
		let short_claim = b"clm".to_vec();
		let long_claim = short_claim.repeat(12);
		let valid_claim = short_claim.repeat(3);
		let sender = 1;
		// negative case for claim length validation
		assert_noop!(
			PoeModule::create_claim(Origin::signed(sender), short_claim),
			Error::<Test>::ProofTooShort
		);
		assert_noop!(
			PoeModule::create_claim(Origin::signed(sender), long_claim),
			Error::<Test>::ProofTooLong
		);

		assert_ok!(PoeModule::create_claim(Origin::signed(sender), valid_claim.clone()));
		// negative case for repeat creation
		assert_noop!(
			PoeModule::create_claim(Origin::signed(sender), valid_claim),
			Error::<Test>::ProofAlreadyClaimed
		);
	});
}

#[test]
fn revoke_claim_should_work() {
	new_test_ext().execute_with(|| {
		let claim = b"claim".to_vec();
		let sender = 1;
		// !IMPORTANT block number is 0 by default, events only emit when block number is greater than 0
		System::set_block_number(1);
		// postive case
		assert_ok!(PoeModule::create_claim(Origin::signed(sender), claim.clone()));
		assert_ok!(PoeModule::revoke_claim(Origin::signed(sender), claim.clone()));

		// verify storage
		assert_ne!(
			Proofs::<Test>::get(&claim),
			(sender, system::Pallet::<Test>::block_number())
		);

		// capture events and compare
		assert_eq!(
			last_event(),
			Event::PoeModule(crate::Event::ClaimRevoked(sender, claim))
		)
	});
}

#[test]
fn revoke_false_claim_should_fail() {
	new_test_ext().execute_with(|| {
		let claim = b"claim".to_vec();
		let sender = 1;
		let hacker = 2; 
		// postive case
		assert_ok!(PoeModule::create_claim(Origin::signed(sender), claim.clone()));
		// not owner can not revoke
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(hacker), claim.clone()),
			Error::<Test>::NotProofOwner
		);

		// can not revoke non-existent claim
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(sender), claim.repeat(2)),
			Error::<Test>::NoSuchProof
		);
	});
}

#[test]
fn transfer_claim_should_work() {
	new_test_ext().execute_with(|| {
		let claim = b"claim".to_vec();
		let sender = 1;
		let receiver = 2;
		// !IMPORTANT block number is 0 by default, events only emit when block number is greater than 0
		System::set_block_number(1);
		// postive case
		assert_ok!(PoeModule::create_claim(Origin::signed(sender), claim.clone()));
		assert_ok!(PoeModule::transfer_claim(Origin::signed(sender), claim.clone(), receiver));

		// verify storage
		assert_eq!(
			Proofs::<Test>::get(&claim),
			(receiver, system::Pallet::<Test>::block_number())
		);

		// capture events and compare
		assert_eq!(
			last_event(),
			Event::PoeModule(crate::Event::ClaimTransferred(sender, receiver,claim))
		)
	});
}

#[test]
fn transfer_claim_should_fail() {
	new_test_ext().execute_with(|| {
		let claim = b"claim".to_vec();
		let sender = 1;
		let receiver = 2; 
		let hacker = 3;
		// postive case
		assert_ok!(PoeModule::create_claim(Origin::signed(sender), claim.clone()));
		// not owner can not transfer
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(hacker), claim.clone(), receiver),
			Error::<Test>::NotProofOwner
		);

		// can not transfer non-existent claim
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(sender), claim.repeat(2), receiver),
			Error::<Test>::NoSuchProof
		);
	});
}