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
fn create_kitty_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // verify storage
		assert_eq!(
			Creator::<Test>::get(0),
            Some(sender)
		);
        // make sure the balance has been reserved
        assert_eq!(Balances::reserved_balance(sender), 20);
        // capture events and compare
		assert_eq!(
			last_event(),
			Event::KittiesModule(crate::Event::KittyCreate(sender, 0))
		)
    })
}

#[test]
fn create_kitty_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 9;
        assert_noop!(
            KittiesModule::create(Origin::signed(sender)),
			Error::<Test>::InsufficientBalanceToReserve
		);
        // emulate kitty id overflow
        KittiesCount::<Test>::put(u32::max_value());
        let valid_send = 1;
        assert_noop!(
            KittiesModule::create(Origin::signed(valid_send)),
			Error::<Test>::KittiesCountOverflow
		);
        // verify storage
		assert_eq!(
			Creator::<Test>::get(0),
            None
		);
    })
}

#[test]
fn buy_kitty_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        assert_eq!(
			Balances::reserved_balance(sender),
            20
		);
        // !IMPORTANT A lazy way to activate the system account, suggest to use `make_free_balance_be` in genesis
        let _ = Balances::transfer(Origin::signed(sender), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        assert_eq!(
			Creator::<Test>::get(0),
            None
		);
        assert_eq!(
			Owner::<Test>::get(0),
            Some(sender)
		);
        assert_eq!(
			last_event(),
			Event::KittiesModule(crate::Event::KittyBuy(sender, 0, 20))
		);
    })
}

#[test]
fn buy_kitty_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let invalid_sender = 2;
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        assert_noop!(
            KittiesModule::buy(Origin::signed(invalid_sender), 0),
			Error::<Test>::NotCreator
		);
        assert_noop!(
            KittiesModule::buy(Origin::signed(sender), 0),
			Error::<Test>::RepatriateFailed
		);
    })
}

#[test]
fn transfer_kitty_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let beneficiary = 2;
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // activate the system's account
        let _ = Balances::transfer(Origin::signed(sender), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        assert_eq!(
			Owner::<Test>::get(0),
            Some(sender)
		);
        // invoke transfer and check the new ownership
        assert_ok!(KittiesModule::transfer(Origin::signed(sender), beneficiary, 0));
        assert_eq!(
			last_event(),
			Event::KittiesModule(crate::Event::KittyTransfer(sender, beneficiary, 0))
		);
        assert_eq!(
			Owner::<Test>::get(0),
            Some(beneficiary)
		);
    })
}

#[test]
fn transfer_kitty_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let beneficiary = 2;
        let hacker = 3;
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // activate the system's account
        let _ = Balances::transfer(Origin::signed(sender), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        assert_eq!(
			Owner::<Test>::get(0),
            Some(sender)
		);
        // hacker try to transfer should fail
        assert_noop!(
            KittiesModule::transfer(Origin::signed(hacker), beneficiary, 0),
			Error::<Test>::NotOwner
		);
    })
}

#[test]
fn sell_kitty_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 1;
        let price = 20;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // activate the system's account
        let _ = Balances::transfer(Origin::signed(4), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        
        // sell the kitty and check balance
        assert_ok!(KittiesModule::sell(Origin::signed(sender), 0, price));
        assert_eq!(Balances::free_balance(sender), 100);
        assert_eq!(
			Owner::<Test>::get(0),
            Some(KittiesModule::account_id())
		);
        assert_eq!(
			last_event(),
			Event::KittiesModule(crate::Event::KittySell(sender, 0, price))
		)
    })
}

#[test]
fn sell_kitty_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 1;
        let hacker = 2;
        let price = 21;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // activate the system's account
        let _ = Balances::transfer(Origin::signed(4), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        
        assert_noop!(
            // we only invalid the invoker
            KittiesModule::sell(Origin::signed(hacker), 0, price - 1),
			Error::<Test>::NotOwner
		);
        assert_noop!(
            KittiesModule::sell(Origin::signed(sender), 0, price),
			Error::<Test>::PriceTooHigh
		);
    })
}


#[test]
fn breed_kitty_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // activate the system's account
        let _ = Balances::transfer(Origin::signed(4), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 1));
        
        assert_ok!(KittiesModule::breed(Origin::signed(sender), 0, 1));
        // capture events and compare
		assert_eq!(
			last_event(),
			Event::KittiesModule(crate::Event::KittyCreate(sender, 2))
		)
    })
}


#[test]
fn breed_kitty_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let sender = 1;
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        assert_ok!(KittiesModule::create(Origin::signed(sender)));
        // activate the system's account
        let _ = Balances::transfer(Origin::signed(4), KittiesModule::account_id(), 1);
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 0));
        assert_ok!(KittiesModule::buy(Origin::signed(sender), 1));
        
        assert_noop!(
            KittiesModule::breed(Origin::signed(sender), 0, 0),
			Error::<Test>::SameParentIndex
		);
        assert_noop!(
            KittiesModule::breed(Origin::signed(sender), 0, 2),
			Error::<Test>::InvalidKittyIndex
		);
    })
}
