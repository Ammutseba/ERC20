#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		type Error = MyError<T>;

		//Generate token
		#[weight = 10_000]
		fn mint(origin, name: Vec<u8>, ticker: Vec<u8>, supply: u64, decimals: u8) -> DispatchResult {
			let creator = ensure_signed(origin)?;
			ensure!(name.len() <= 64, MyError::<T>::NameTooBig);
			ensure!(ticker.len() <= 32, MyError::<T>::TickerTooBig);
			ensure!(Self::get_mint() == false, MyError::<T>::AlreadyMinted);
			
			Ticker::put(ticker);
			Name::put(name);
			MaxSupply::put(supply);
			Decimals::put(decimals);	

			<Balances<T>>::insert(creator, supply);
			Minted::put(true);

			Ok(())
		}

		//Returns the name of the token
		#[weight = 10_000]
		fn name(origin) -> DispatchResult {
			let _user = ensure_signed(origin)?;
			let name = Self::get_name();
			Self::deposit_event(RawEvent::NameReturned(name));
			Ok(())
		}

		//Returns the symbol of the token. E.g. “HIX”.
		#[weight = 10_000]
		fn symbol(origin) -> DispatchResult {
			let _user = ensure_signed(origin)?;
			let ticker = Self::get_ticker();
			Self::deposit_event(RawEvent::TickerReturned(ticker));
			Ok(())
		}

		//Returns the number of decimals the token uses - e.g. 8, means to divide the token amount by 100000000 to get its user representation. Default is 18.
		#[weight = 10_000]
		fn decimals(origin) -> DispatchResult {
			let _user = ensure_signed(origin)?;
			let decimals = Self::get_decimals();
			Self::deposit_event(RawEvent::DecimalsReturned(decimals));
			Ok(())
		}

		//Returns the total token supply.
		#[weight = 10_000]
		fn total_supply(origin) -> DispatchResult {
			let _user = ensure_signed(origin)?;
			let max_supply = Self::get_max_supply();
			Self::deposit_event(RawEvent::TotalSupplyReturned(max_supply));
			Ok(())
		}

		//Returns the account balance of an account
		#[weight = 10_000]
		fn balance_of(origin) -> DispatchResult {
			let user = ensure_signed(origin)?;
			ensure!(<Balances<T>>::contains_key(&user), MyError::<T>::NoValueStored);
			let owner_original_value = <Balances<T>>::get(&user);

			Self::deposit_event(RawEvent::BalanceReturned(owner_original_value));
			Ok(())
		}

		//Transfers value amount of tokens from origin to 'to'
		#[weight = 10_000]
		fn transfer(origin, to: T::AccountId, value: u64) -> DispatchResult {
			let user = ensure_signed(origin)?;
			ensure!(<Balances<T>>::contains_key(&user), MyError::<T>::NoValueStored);
			let owner_original_value = <Balances<T>>::get(&user);
			ensure!(owner_original_value >= value, MyError::<T>::NotEnoughFunds);
			let receiver_original_value = <Balances<T>>::get(&to);

			let owner_resulting_value = owner_original_value - value;
			let receiver_resulting_value = receiver_original_value + value;
			
			<Balances<T>>::insert(&user, owner_resulting_value);
			<Balances<T>>::insert(&to, receiver_resulting_value);

			Self::deposit_event(RawEvent::Transfer(user, to, value));
			Ok(())
		}

		// Transfers value amount of tokens from address 'from' to address 'to' depending on the allowance
		#[weight = 10_000]
		fn transfer_from(origin, from: T::AccountId, to: T::AccountId, value: u64) -> DispatchResult {
			let _user = ensure_signed(origin)?;

			let allowance = <Allowances<T>>::get(&from, &to);
			ensure!(allowance >= value, MyError::<T>::NotEnoughAllowance);
			let updated_allowance = allowance - value ;
			ensure!(<Balances<T>>::contains_key(&from), MyError::<T>::NoValueStored);
			let owner_original_value = <Balances<T>>::get(&from);
			ensure!(owner_original_value >= value, MyError::<T>::NotEnoughFunds);
			let receiver_original_value = <Balances<T>>::get(&to);
		
			let owner_resulting_value = owner_original_value - value;
			let receiver_resulting_value = receiver_original_value + value;

			<Allowances<T>>::insert(&from, &to, updated_allowance);				
			<Balances<T>>::insert(&from, owner_resulting_value);
			<Balances<T>>::insert(&to, receiver_resulting_value);

			Self::deposit_event(RawEvent::Transfer(from, to, value));
			Ok(())
		}

		//Allows spender to withdraw from your account multiple times, up to the value amount.
		//If this function is called again it overwrites the current allowance with value.
		#[weight = 10_000]
		fn approve(origin, to: T::AccountId, value: u64) -> DispatchResult {
			let user = ensure_signed(origin)?;

			<Allowances<T>>::insert(&user, &to, value);

			Self::deposit_event(RawEvent::Approval(user, to, value));
			Ok(())
		}

		//Returns the amount which spender is still allowed to withdraw from owner.
		#[weight = 10_000]
		fn allowance(origin, to: T::AccountId) -> DispatchResult {
			let user = ensure_signed(origin)?;
			ensure!(<Allowances<T>>::contains_key(&user, &to), MyError::<T>::NoValueStored);

			let value = <Allowances<T>>::get(&user, &to);

			Self::deposit_event(RawEvent::AllowanceReturned(value));
			Ok(())
		}
	}
}

decl_storage! {
	trait Store for Module<T: Config> as TokenStorage {
		pub MaxSupply get(fn get_max_supply): u64;
		pub Decimals get(fn get_decimals): u8 = 18;
		pub Ticker get(fn get_ticker): Vec::<u8>;
		pub Minted get(fn get_mint): bool = false;
		pub Name get(fn get_name): Vec::<u8>;
		pub Balances get(fn balances): map hasher(blake2_128_concat) T::AccountId => u64;
		pub Allowances get(fn allowances): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => u64;
	}
}

decl_error! {
	pub enum MyError for Module<T: Config> {
		TickerTooBig,
		NameTooBig,
		NoValueStored,
		NotEnoughFunds,
		AlreadyMinted,
		NotEnoughAllowance,
	}
}

decl_event! (
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		NameReturned(Vec::<u8>),
		TickerReturned(Vec::<u8>),
		DecimalsReturned(u8),
		Minted(bool),
		TotalSupplyReturned(u64),
		BalanceReturned(u64),
		Transfer(AccountId, AccountId, u64),
		Approval(AccountId, AccountId, u64),
		AllowanceReturned(u64),
	}
);
