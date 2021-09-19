#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_support::traits::{Randomness, Currency, ReservableCurrency, ExistenceRequirement};
	use frame_system::pallet_prelude::*;
	use codec::{Encode, Decode};
	use sp_io::hashing::blake2_128;

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8;16]);

	type KittyIndex = u32;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
    	type StakeForEachKitty: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, KittyIndex),
		Sell(T::AccountId, KittyIndex, Option<BalanceOf<T>>),
		Bought(T::AccountId, T::AccountId, KittyIndex, Option<BalanceOf<T>>),
	}

	#[pallet::storage]
    #[pallet::getter(fn get_nonce)]
    pub(super) type Nonce<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<T::AccountId>, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn kitties_price)]
	pub type KittiesPrice<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
		NotForSale,
		NotEnoughForBuying,
		NotEnoughForStaking,
		BuyerIsOwner,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::kitties_count();
			ensure!(kitty_id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);

			let dna = Self::random_value(&who);

			let stake = T::StakeForEachKitty::get();

			T::Currency::reserve(&who, stake)
                .map_err(|_| Error::<T>::NotEnoughForStaking)?;

			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));

			Owner::<T>::insert(kitty_id, Some(who.clone()));

			KittiesCount::<T>::put(kitty_id + 1);

			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: KittyIndex) ->
			DispatchResult
		{
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			Self::transfer_from(who, new_owner, kitty_id)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex)
			-> DispatchResult
		{
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = Self::kitties_count();
			ensure!(kitty_id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			let stake = T::StakeForEachKitty::get();

			T::Currency::reserve(&who, stake)
                .map_err(|_| Error::<T>::NotEnoughForStaking)?;

			Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));

			Owner::<T>::insert(kitty_id, Some(who.clone()));

			KittiesCount::<T>::put(kitty_id + 1);

			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn sell(origin: OriginFor<T>, kitty_id: KittyIndex, price: Option<BalanceOf<T>>)
			-> DispatchResult 
		{
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			KittiesPrice::<T>::mutate_exists(kitty_id, |p| *p = Some(price));

			Self::deposit_event(Event::Sell(who, kitty_id, price));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyIndex)
			-> DispatchResult 
		{
			let who = ensure_signed(origin)?;

			let owner = Owner::<T>::get(kitty_id).ok_or(Error::<T>::NotOwner)?;

			ensure!(Some(who.clone()) != Some(owner.clone()), Error::<T>::BuyerIsOwner);

			let kitty_price = KittiesPrice::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;

			let balance = T::Currency::free_balance(&who);
            
            let stake = T::StakeForEachKitty::get();

			ensure!(balance > (kitty_price + stake), Error::<T>::NotEnoughForBuying);

			T::Currency::reserve(&who, stake)
                .map_err(|_| Error::<T>::NotEnoughForStaking)?;
            
			T::Currency::unreserve(&owner, stake);

			T::Currency::transfer(
                &who,
                &owner,
                kitty_price,
                ExistenceRequirement::KeepAlive,
            )?;

			Self::transfer_from(owner.clone(), who.clone(), kitty_id)?;

			KittiesPrice::<T>::remove(kitty_id);

			Self::deposit_event(Event::Bought(who, owner, kitty_id, Some(kitty_price)));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: KittyIndex) -> DispatchResult 
		{
			let stake = T::StakeForEachKitty::get();

			T::Currency::reserve(&to, stake)
                .map_err(|_| Error::<T>::NotEnoughForStaking)?;

            T::Currency::unreserve(&from, stake);

			Owner::<T>::insert(kitty_id, Some(to.clone()));

			Self::deposit_event(Event::KittyTransfer(from, to, kitty_id));

			Ok(())
		}
	}
}