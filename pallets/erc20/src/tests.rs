use crate::{mock::*, Error, Event};
use frame_support::{assert_err, assert_ok};

#[test]
fn inititates_correctly() {
	ExtBuilder::build().execute_with(|| {
		let alice = sp_runtime::MultiSigner::from(Alice.public()).into_account();

		let alice_balance = Erc20::balance_of(alice);
		assert_eq!(Erc20::total_supply(), 20000000);
		assert_eq!(alice_balance, 20000000);
	});
}

#[test]
fn emits_transfer_event() {
	ExtBuilder::build().execute_with(|| {
		let alice = sp_runtime::MultiSigner::from(Alice.public()).into_account();
		let bob = sp_runtime::MultiSigner::from(Bob.public()).into_account();

		let _transfer = Erc20::transfer(RuntimeOrigin::signed(alice.clone()), bob.clone(), 200);
		System::assert_has_event(RuntimeEvent::from(Event::Tranferred {
			from: alice,
			to: bob,
			value: 200,
		}));
	});
}
#[test]
fn right_transfer_balances() {
	ExtBuilder::build().execute_with(|| {
		let alice = sp_runtime::MultiSigner::from(Alice.public()).into_account();
		let bob = sp_runtime::MultiSigner::from(Bob.public()).into_account();

		let _transfer = Erc20::transfer(RuntimeOrigin::signed(alice.clone()), bob.clone(), 200);
		assert_eq!(Erc20::balance_of(alice), 19999800);
		assert_eq!(Erc20::balance_of(bob), 200);
	});
}

#[test]
fn fails_without_approve() {
	ExtBuilder::build().execute_with(|| {
		let alice = sp_runtime::MultiSigner::from(Alice.public()).into_account();
		let bob = sp_runtime::MultiSigner::from(Bob.public()).into_account();
		let charlie = sp_runtime::MultiSigner::from(Charlie.public()).into_account();

		let transfer_from = Erc20::transfer_from(
			RuntimeOrigin::signed(bob.clone()),
			alice.clone(),
			charlie.clone(),
			200,
		);
		assert_err!(transfer_from, Error::<Test>::ERC20InsufficientAllowance);
	})
}

#[test]
fn ok_after_approve() {
	ExtBuilder::build().execute_with(|| {
		let alice = sp_runtime::MultiSigner::from(Alice.public()).into_account();
		let bob = sp_runtime::MultiSigner::from(Bob.public()).into_account();

		let approve = Erc20::approve(RuntimeOrigin::signed(alice.clone()), bob, 200);
		assert_ok!(approve);
	})
}
