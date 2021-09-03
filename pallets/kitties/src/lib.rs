#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
        dispatch::DispatchResult, PalletId, pallet_prelude::*, 
        traits::{Randomness, Currency, ReservableCurrency, BalanceStatus, ExistenceRequirement::{AllowDeath}}
    };
	use frame_system::pallet_prelude::*;
	use codec::{Encode, Decode};

	use sp_io::hashing::blake2_128;
    use sp_runtime::{
        traits::{
            Zero, AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, One, AccountIdConversion
        },
    };
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8;16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type Currency: ReservableCurrency<Self::AccountId>;
        // Refer https://github.com/paritytech/substrate/blob/49a4103f4bfef55be20a5c6d26e18ff3003c3353/frame/balances/src/lib.rs#L193
        type KittyIndex: Parameter + Member + AtLeast32BitUnsigned  + Default + Copy + MaybeSerializeDeserialize;
        /// The amount of token to be reserved when creating a kitty
        type BalanceToReserve: Get<BalanceOf<Self>>;
        #[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, T::KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
        KittyBuy(T::AccountId, T::KittyIndex, BalanceOf<T>),
        KittySell(T::AccountId, T::KittyIndex, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

    #[pallet::storage]
	#[pallet::getter(fn creator)]
	pub type Creator<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;


	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
        NotCreator,
		SameParentIndex,
		InvalidKittyIndex,
        InsufficientBalanceToReserve,
        RepatriateFailed,
        PriceTooHigh,
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
            let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => Zero::zero()
			};
            // reserve the caller's balance
            let _ = T::Currency::reserve(&who, T::BalanceToReserve::get()).map_err(|_| Error::<T>::InsufficientBalanceToReserve)?;
			let dna = Self::random_value(&who);
            // Make sure the created kitty can not be sold/transferred
			Self::gen_kitty(kitty_id, Self::account_id(), dna);
            // This is to make sure only creator can buy its kitty
            Creator::<T>::insert(kitty_id, Some(who.clone()));

			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) ->
			DispatchResult
		{
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex)
			-> DispatchResult
		{
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

				let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => Zero::zero()
			};

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			Self::gen_kitty(kitty_id, who.clone(), new_dna);
			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(())
		}
        #[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Some(who.clone()) == Creator::<T>::get(kitty_id), Error::<T>::NotCreator);
            // move the reserved balance to system's account free balance so that sell can proceed
            let amount = T::BalanceToReserve::get();
            let _ = T::Currency::repatriate_reserved(
                &who, &Self::account_id(), amount, BalanceStatus::Free
            ).map_err(|_| Error::<T>::RepatriateFailed)?;
            // update owner and remove creator
            Owner::<T>::insert(kitty_id, Some(who.clone()));
            Creator::<T>::remove(kitty_id);

            Self::deposit_event(Event::KittyBuy(who, kitty_id, amount));

            Ok(())
        }

        #[pallet::weight(0)]
		pub fn sell(origin: OriginFor<T>, kitty_id: T::KittyIndex, #[pallet::compact] price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
            // get system's free balance
            let amount = T::Currency::free_balance(&Self::account_id());
            ensure!(amount > price, Error::<T>::PriceTooHigh);

            let _ = T::Currency::transfer(&Self::account_id(), &who, price, AllowDeath);
            Owner::<T>::insert(kitty_id, Some(Self::account_id()));
            Self::deposit_event(Event::KittySell(who, kitty_id, price));

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
        
        /// refer https://github.com/paritytech/substrate/blob/743accbe3256de2fc615adcaa3ab03ebdbbb4dbd/frame/treasury/src/lib.rs#L351
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        /// create a kitty and bind with owner, also update the global count
        fn gen_kitty(kitty_id: T::KittyIndex, owner: T::AccountId, dna: [u8; 16]) {
            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			Owner::<T>::insert(kitty_id, Some(owner));
			KittiesCount::<T>::put(kitty_id + One::one());
        }
	}
}
