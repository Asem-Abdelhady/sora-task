#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::{OriginFor, *};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	//

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Tranferred { from: T::AccountId, to: T::AccountId, value: u64 },
		Approved { owner: T::AccountId, spender: T::AccountId, value: u64 },
		Initalized { who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		ERC20InsufficientBalance,
		ERC20InvalidSender,
		ERC20InvalidReceiver,
		ERC20InsufficientAllowance,
		ERC20InvalidApprover,
		ERC20InvalidSpender,
		AlreadyInialized,
		NoTokenInAccount,
		NegativeTotalSupply,
	}

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type BalanceOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type Allowences<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, T::AccountId), u64, ValueQuery>;

	#[pallet::type_value]
	pub(super) fn TotalSupplyDefaultValue<T: Config>() -> u64 {
		20000000
	}
	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub(super) type TotalSupply<T: Config> =
		StorageValue<_, u64, ValueQuery, TotalSupplyDefaultValue<T>>;

	#[pallet::storage]
	pub(super) type IsInitialized<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		#[pallet::call_index(0)]
		pub fn init(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(!<IsInitialized<T>>::get(), <Error<T>>::AlreadyInialized);

			<BalanceOf<T>>::insert(sender.clone(), <TotalSupply<T>>::get());

			Self::deposit_event(Event::Initalized { who: sender });
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		#[pallet::call_index(1)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			value: u64,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let sender_balance = <BalanceOf<T>>::get(&sender);
			let receiver_balance = <BalanceOf<T>>::get(&to);

			let updated_from_balance =
				sender_balance.checked_sub(value).ok_or(<Error<T>>::ERC20InsufficientBalance)?;
			let updated_to_balance =
				receiver_balance.checked_add(value).expect("Doesn't fit the u64");
			<BalanceOf<T>>::insert(&sender, updated_from_balance);
			<BalanceOf<T>>::insert(&to, updated_to_balance);

			Self::deposit_event(Event::Tranferred { from: sender, to, value });
			Ok(().into())
		}
	}
	impl<T: Config> Pallet<T> {
		fn burn(from: &T::AccountId, amount: u64) -> Result<(), Error<T>> {
			let burner_balance = Self::balance_of(from);
			if burner_balance > amount {
				let updated_from_balance = burner_balance
					.checked_sub(amount)
					.ok_or(<Error<T>>::ERC20InsufficientBalance)?;
				let total_supply = Self::total_supply();
				let updated_from_total_supply =
					total_supply.checked_sub(amount).ok_or(<Error<T>>::NegativeTotalSupply)?;
				<BalanceOf<T>>::insert(from, updated_from_balance);
				<TotalSupply<T>>::put(updated_from_total_supply);
				Ok(())
			} else {
				Err(<Error<T>>::NoTokenInAccount)
			}
		}

		fn approve_spender(
			owner: &T::AccountId,
			spender: &T::AccountId,
			amount: value,
		) -> Result<(), Error<T>> {
		}
	}
}
