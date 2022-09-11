#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		ensure,
		pallet_prelude::*,
		sp_runtime::traits::{AtLeast32BitUnsigned, Bounded},
		traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;

	#[derive(Encode, Decode, TypeInfo)]
	pub struct Kitty {
		pub dna: [u8; 16],
	}

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_list_for_sales)]
	pub type ListForSale<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded;

		#[pallet::constant]
		type StakeForEachKitty: Get<BalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, T::KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
		KittyListed(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		KittyTrade(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
		BuyerIsOwner,
		KittyNotForSell,
		NotEnoughBalanceForBuying,
		NotEnoughBalanceForStaking,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let dna = Self::random_value(&who);
			Self::mint(&who, dna)
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, parent1_kitty_id: T::KittyIndex, parent2_kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(parent1_kitty_id != parent2_kitty_id, Error::<T>::SameParentIndex);

			let parent1_kitty = Self::kitties(parent1_kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			let parent2_kitty = Self::kitties(parent2_kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			let parent1_dna = parent1_kitty.dna;
			let parent2_dna = parent2_kitty.dna;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];
			for i in 0..parent1_dna.len() {
				new_dna[i] = (selector[i] & parent1_dna[i]) | (!selector[i] & parent2_dna[i]);
			}
			Self::mint(&who, new_dna)
		}

		#[pallet::weight(0)]
		pub fn sell(origin: OriginFor<T>, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			ListForSale::<T>::insert(kitty_id, price);
			Self::deposit_event(Event::KittyListed(who, kitty_id, price));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			let stake_amount = T::StakeForEachKitty::get();

			T::Currency::reserve(&new_owner, stake_amount).map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

			T::Currency::unreserve(&who, stake_amount);

			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let seller = Owner::<T>::get(kitty_id).unwrap();
			ensure!(Some(buyer.clone()) != Some(seller.clone()), Error::<T>::BuyerIsOwner);

			let kitty_price = ListForSale::<T>::get(kitty_id).ok_or(Error::<T>::KittyNotForSell)?;
			let buyer_balance = T::Currency::free_balance(&buyer);
			let stake_amount = T::StakeForEachKitty::get();

			ensure!(buyer_balance > (kitty_price + stake_amount),Error::<T>::NotEnoughBalanceForBuying);

			T::Currency::reserve(&buyer, stake_amount).map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;
			T::Currency::unreserve(&seller, stake_amount);

			T::Currency::transfer(&buyer, &seller, kitty_price, ExistenceRequirement::KeepAlive)?;
			Owner::<T>::insert(kitty_id, Some(buyer.clone()));

			ListForSale::<T>::remove(kitty_id);
			Self::deposit_event(Event::KittyTrade(buyer, seller, kitty_id));
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

		fn mint(owner: &T::AccountId, dna: [u8; 16]) -> DispatchResult {
			let kitty_id = match Self::kitty_cnt() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				}
				None => 0u32.into(),
			};

			let stake_amount = T::StakeForEachKitty::get();
			T::Currency::reserve(&owner, stake_amount).map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

			Kitties::<T>::insert(kitty_id, Some(Kitty { dna }));
			Owner::<T>::insert(kitty_id, Some(owner.clone()));
			KittyCount::<T>::put(kitty_id + 1u32.into());

			log::info!("The kitty_id is: {:?}.", kitty_id);
			Self::deposit_event(Event::KittyCreate(owner.clone(), kitty_id));
			Ok(())
		}
	}
}
