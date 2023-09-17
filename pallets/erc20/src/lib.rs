#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{DispatchResultWithPostInfo, *},
		sp_runtime,
		traits::BuildGenesisConfig,
		Blake2_128Concat,
	};
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
	}

	#[pallet::error]
	pub enum Error<T> {
		ERC20InsufficientBalance,
		ERC20InsufficientAllowance,
		ERC20InvalidApprover,
		NoTokenInAccount,
		NegativeTotalSupply,
	}

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type BalanceOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allowance)]

	pub(super) type Allowances<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, T::AccountId), u64, ValueQuery>;

	#[pallet::type_value]
	pub(super) fn TotalSupplyDefaultValue<T: Config>() -> u64 {
		20000000
	}
	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub(super) type TotalSupply<T: Config> =
		StorageValue<_, u64, ValueQuery, TotalSupplyDefaultValue<T>>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub total_supply: u64,
		pub supply_owner: Option<T::AccountId>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			TotalSupply::<T>::put(self.total_supply);
			if let Some(ref supply_owner) = self.supply_owner {
				BalanceOf::<T>::insert(supply_owner, self.total_supply);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		#[pallet::call_index(0)]
		pub fn approve(
			owner: OriginFor<T>,
			spender: T::AccountId,
			value: u64,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(owner)?;
			ensure!(spender != signer, <Error<T>>::ERC20InvalidApprover);
			let owner_balance = Self::balance_of(&signer);
			ensure!(owner_balance > value, <Error<T>>::ERC20InsufficientBalance);
			<Allowances<T>>::insert((&signer, &spender), value);
			Self::deposit_event(Event::Approved { owner: signer, spender, value });

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		#[pallet::call_index(1)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			value: u64,
		) -> DispatchResultWithPostInfo {
			let spender = ensure_signed(origin)?;
			let spender_balance = Self::balance_of(&spender);
			ensure!(spender_balance > 0, <Error<T>>::ERC20InsufficientBalance);
			Self::_transfer(&spender, &to, value, None)?;

			Self::deposit_event(Event::Tranferred { from: spender, to, value });
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		#[pallet::call_index(2)]
		pub fn transfer_from(
			spender: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(spender)?;
			let spender_balance = Self::balance_of(&signer);
			let owner_balance = Self::balance_of(&from);
			ensure!(spender_balance > 0, <Error<T>>::ERC20InsufficientAllowance);
			ensure!(owner_balance > 0, <Error<T>>::ERC20InsufficientBalance);
			Self::_transfer(&from, &to, value, Some((&from, &signer)))?;
			Self::deposit_event(Event::Tranferred { from: signer, to, value });

			Ok(().into())
		}
	}
	impl<T: Config> Pallet<T> {
		// The next function is not called
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

		fn _transfer(
			sender: &T::AccountId,
			receiver: &T::AccountId,
			value: u64,
			from_allowance: Option<(&T::AccountId, &T::AccountId)>,
		) -> Result<(), Error<T>> {
			let sender_balance = <BalanceOf<T>>::get(sender);
			let receiver_balance = <BalanceOf<T>>::get(receiver);

			let updated_from_balance = Self::_checked_sub(sender_balance, value)?;
			let updated_receiver_balance = Self::_checked_add(receiver_balance, value)?;

			<BalanceOf<T>>::insert(receiver, updated_receiver_balance);
			<BalanceOf<T>>::insert(sender, updated_from_balance);

			match from_allowance {
				Some(allowance) => {
					let current_allowance = <Allowances<T>>::get(allowance);
					let updated_current_allowance = Self::_checked_sub(current_allowance, value)?;
					<Allowances<T>>::insert(allowance, updated_current_allowance);
					Ok(())
				},
				None => Ok(()),
			}
		}

		fn _checked_sub(current_value: u64, mius_value: u64) -> Result<u64, Error<T>> {
			let updated_value = current_value
				.checked_sub(mius_value)
				.ok_or(<Error<T>>::ERC20InsufficientBalance)?;
			Ok(updated_value)
		}

		fn _checked_add(current_value: u64, added_value: u64) -> Result<u64, Error<T>> {
			let updated_value =
				current_value.checked_add(added_value).expect("Doesn't fit the u64");
			Ok(updated_value)
		}
	}
}
