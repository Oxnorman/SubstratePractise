use frame_support::{assert_noop, assert_ok};

use crate::mock::{
	Event as TestEvent, KittiesModule, new_test_ext, Origin, System, Test,
};

use super::*;

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let kitty_id: u32 = 0;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_eq!(Owner::<Test>::get(kitty_id), Some(account_id));
		assert_has_event!(Event::<Test>::KittyCreate(account_id, kitty_id));
	});
}

#[test]
fn create_kitty_failed_count_overflow() {
	new_test_ext().execute_with(|| {
		KittyCount::<Test>::put(u32::MAX);
		let account_id = 1;
		assert_noop!(KittiesModule::create(Origin::signed(account_id)),Error::<Test>::KittiesCountOverflow);
	});
}

//
#[test]
fn create_kitty_failed_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 100;
		assert_noop!(KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::NotEnoughBalanceForStaking);
	});
}

#[test]
fn breed_works() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let parent1_kitty_id = 0u32;
		let parent2_kitty_id = 1u32;
		let new_kitty_id: u32 = 2u32;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_ok!(KittiesModule::breed(Origin::signed(account_id), parent1_kitty_id, parent2_kitty_id));
		assert_has_event!(Event::<Test>::KittyCreate(account_id, new_kitty_id));
	});
}

#[test]
fn breed_failed_same_parent() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let parent1_kitty_id = 0u32;
		let parent2_kitty_id = 0u32;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_noop!(KittiesModule::breed(Origin::signed(account_id),
			parent1_kitty_id,
			parent2_kitty_id),
			Error::<Test>::SameParentIndex);
	});
}

#[test]
fn breed_failed_invalid_kitty_index() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let parent1_kitty_id = 0u32;
		let parent2_kitty_id = 1u32;
		assert_noop!(KittiesModule::breed(Origin::signed(account_id),
			parent1_kitty_id,
			parent2_kitty_id),
			Error::<Test>::InvalidKittyIndex);
	});
}

//
#[test]
fn breed_failed_count_overflow() {
	new_test_ext().execute_with(|| {
		KittyCount::<Test>::put(u32::MAX - 2);
		let account_id: u64 = 1;
		let parent1_kitty_id = u32::MAX - 1;
		let parent2_kitty_id = u32::MAX - 2;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_noop!(KittiesModule::breed(Origin::signed(account_id),
			parent1_kitty_id,
			parent2_kitty_id),
			Error::<Test>::KittiesCountOverflow);
	});
}

#[test]
fn breed_failed_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// 4 没有分配 token
		let other_account_id: u64 = 4;
		let parent1_kitty_id = 0;
		let parent2_kitty_id = 1;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_noop!(KittiesModule::breed(Origin::signed(other_account_id),
			parent1_kitty_id,
			parent2_kitty_id),
			Error::<Test>::NotEnoughBalanceForStaking);
	});
}

#[test]
fn sell_works() {
	new_test_ext().execute_with(|| {
		let account_seller: u64 = 1;
		let kitty_id = 0;
		let price = 100;
		assert_ok!(KittiesModule::create(Origin::signed(account_seller)));
		assert_ok!(KittiesModule::sell(Origin::signed(account_seller), kitty_id, Some(price)));
		// 代售列表中的价格是否一致
		assert_eq!(ListForSale::<Test>::get(kitty_id), Some(price));
		// 卖出事件
		assert_has_event!(Event::<Test>::KittyListed(account_seller, kitty_id, Some(price)));
	});
}

#[test]
fn sell_failed_not_owner() {
	new_test_ext().execute_with(|| {
		let account_owner: u64 = 1;
		let account_seller: u64 = 2;
		let kitty_id = 0;
		let price = 100;
		assert_ok!(KittiesModule::create(Origin::signed(account_owner)));
		assert_noop!(KittiesModule::sell(Origin::signed(account_seller), kitty_id, Some(price)),Error::<Test>::NotOwner);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let account_giver: u64 = 1;
		let account_receiver: u64 = 2;
		let kitty_id = 0;
		assert_ok!(KittiesModule::create(Origin::signed(account_giver)));
		assert_ok!(KittiesModule::transfer(Origin::signed(account_giver), account_receiver, kitty_id));
		assert_has_event!(Event::KittyTransfer(account_giver, account_receiver, kitty_id));
	});
}

#[test]
fn transfer_failed_not_owner() {
	new_test_ext().execute_with(|| {
		let account_owner: u64 = 1;
		let account_seller: u64 = 2;
		let account_buyer: u64 = 3;
		let kitty_id = 0;
		assert_ok!(KittiesModule::create(Origin::signed(account_owner)));
		assert_noop!(KittiesModule::transfer(Origin::signed(account_seller), account_buyer, kitty_id),Error::<Test>::NotOwner);
	});
}

#[test]
fn transfer_failed_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let account_seller: u64 = 1;
		// 4 没有分配 token
		let account_buyer: u64 = 4;
		let kitty_id = 0;
		assert_ok!(KittiesModule::create(Origin::signed(account_seller)));
		assert_noop!(KittiesModule::transfer(Origin::signed(account_seller), account_buyer, kitty_id),Error::<Test>::NotEnoughBalanceForStaking);
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		let account_seller: u64 = 1;
		let account_buyer: u64 = 2;
		let kitty_id = 0;
		let price: u128 = 100;
		assert_ok!(KittiesModule::create(Origin::signed(account_seller)));
		assert_ok!(KittiesModule::sell(Origin::signed(account_seller), kitty_id, Some(price)));
		assert_ok!(KittiesModule::buy(Origin::signed(account_buyer), kitty_id));
		assert_has_event!(Event::KittyTrade(account_buyer, account_seller, kitty_id));
	});
}

#[test]
fn buy_failed_buyer_is_owner() {
	new_test_ext().execute_with(|| {
		let account_buyer: u64 = 1;
		let kitty_id = 0;
		let price: u128 = 100;
		assert_ok!(KittiesModule::create(Origin::signed(account_buyer)));
		assert_ok!(KittiesModule::sell(Origin::signed(account_buyer), kitty_id, Some(price)));
		assert_noop!(KittiesModule::buy(Origin::signed(account_buyer), kitty_id),Error::<Test>::BuyerIsOwner);
	});
}

#[test]
fn buy_failed_not_for_sell() {
	new_test_ext().execute_with(|| {
		let account_seller: u64 = 1;
		let account_buyer: u64 = 2;
		let kitty_id = 0;
		assert_ok!(KittiesModule::create(Origin::signed(account_seller)));
		assert_noop!(KittiesModule::buy(Origin::signed(account_buyer), kitty_id),Error::<Test>::KittyNotForSell);
	});
}

#[test]
fn buy_failed_buyer_not_enough_balance_for_buying() {
	new_test_ext().execute_with(|| {
		let account_seller: u64 = 1;
		let account_buyer: u64 = 3;
		let kitty_id = 0;
		let price: u128 = 100000;
		assert_ok!(KittiesModule::create(Origin::signed(account_seller)));
		assert_ok!(KittiesModule::sell(Origin::signed(account_seller), kitty_id, Some(price)));
		assert_noop!(KittiesModule::buy(Origin::signed(account_buyer), kitty_id),Error::<Test>::NotEnoughBalanceForBuying);
	});
}
