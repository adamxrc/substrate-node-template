use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_eq!(KittiesCount::<Test>::get(), 1);
		assert_eq!(Owner::<Test>::get(0), Some(1));
	});
}

#[test]
fn create_kitty_not_enough_staking() {
	new_test_ext().execute_with(|| {
		assert_noop!(KittiesModule::create(Origin::signed(3)), Error::<Test>::NotEnoughForStaking);
	});
}

#[test]
fn transfer_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::transfer(Origin::signed(1), 2, 0));
		assert_eq!(Owner::<Test>::get(0), Some(2));
	});
}

#[test]
fn transfer_kitty_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(KittiesModule::transfer(Origin::signed(2), 3, 0), Error::<Test>::NotOwner);
	});
}

#[test]
fn transfer_kitty_not_enough_staking() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(KittiesModule::transfer(Origin::signed(1), 3, 0), Error::<Test>::NotEnoughForStaking);
	});
}

#[test]
fn breed_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(2)));
		assert_ok!(KittiesModule::breed(Origin::signed(1), 0, 1));
		assert_eq!(KittiesCount::<Test>::get(), 3);
		assert_eq!(Owner::<Test>::get(2), Some(1));
	});
}

#[test]
fn breed_kitty_same_parent() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(2)));
		assert_noop!(KittiesModule::breed(Origin::signed(1), 1, 1), Error::<Test>::SameParentIndex);
	});
}

#[test]
fn breed_kitty_not_enough_staking() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(2)));
		assert_noop!(KittiesModule::breed(Origin::signed(3), 0, 1), Error::<Test>::NotEnoughForStaking);
	});
}

#[test]
fn sell_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell(Origin::signed(1), 0, Some(100)));
		assert_eq!(KittiesPrice::<Test>::get(0), Some(100));
	});
}

#[test]
fn sell_kitty_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(KittiesModule::sell(Origin::signed(2), 0, Some(100)), Error::<Test>::NotOwner);
	});
}

#[test]
fn buy_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell(Origin::signed(1), 0, Some(100)));
		assert_ok!(KittiesModule::buy(Origin::signed(2), 0));
		assert_eq!(Owner::<Test>::get(0), Some(2));
	});
}

#[test]
fn buy_kitty_price_not_for_sale() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(KittiesModule::buy(Origin::signed(2), 0), Error::<Test>::NotForSale);
	});
}

#[test]
fn buy_kitty_not_enough_buying() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell(Origin::signed(1), 0, Some(200)));
		assert_noop!(KittiesModule::buy(Origin::signed(2), 0), Error::<Test>::NotEnoughForBuying);
	});
}
