#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

type PetId = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum length of a metadata string.
		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	#[derive(
		Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	pub enum Species {
		#[default]
		Turtle,
		Snake,
		Rabbit,
	}

	#[derive(
		Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	#[scale_info(skip_type_params(T))]
	pub struct PetInfo<T: Config> {
		pub name: BoundedVec<u8, T::StringLimit>,
		pub species: Species,
	}

	/// Onchain storage for pet info.
	#[pallet::storage]
	pub type PetsInfo<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, (PetId, PetInfo<T>)>;

	/// Store the last feed time of a pet, use block number for time reference.
	#[pallet::storage]
	pub type LastFeedTime<T: Config> =
		StorageMap<_, Blake2_128Concat, PetId, T::BlockNumber, ValueQuery>;

	/// Store the last sleep time of a pet, use block number for time reference.
	#[pallet::storage]
	pub type LastSleepTime<T: Config> = StorageMap<_, Blake2_128Concat, PetId, T::BlockNumber>;

	/// Events for this module.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new pet is minted. \[owner, petid\]
		PetMinted(T::AccountId, u32),
		/// Pet is transfered. \[from, to, petid\]
		PetTransfered(T::AccountId, T::AccountId, u32),
		/// Pet is feeded. \[owner, petid\]
		PetFeeded(T::AccountId, u32),
		/// Pet is sleep. \[owner, petid\]
		PetSleeped(T::AccountId, u32),
	}

	/// Errors for this module.
	#[pallet::error]
	pub enum Error<T> {
		AccountAlreadyHasPet,
		AccountHasNoPet,
	}

	/// Dispatchables for this module.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Mint a new pet by reserving a certain mount of token.
		/// One user can have many pets, but one pet can only be owned by one user.
		/// The id of the pet is unique and can be set by its owner.
		///
		/// - name: The name of the pet
		/// - speies: The species of the pet
		/// - id: The id of the pet
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn mint(
			origin: OriginFor<T>,
			name: BoundedVec<u8, T::StringLimit>,
			species: Species,
			id: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(!PetsInfo::<T>::contains_key(&sender), Error::<T>::AccountAlreadyHasPet);

			let pet = PetInfo {
				name,
				species,
			};

			PetsInfo::<T>::insert(&sender, (id, pet));

			Self::deposit_event(Event::PetMinted(sender, id));

			Ok(().into())
		}

		/// Transfer a pet
		///
		/// - receiver: The receiver of the pet
		/// - id: The id of the pet
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			receiver: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (id, pet) = PetsInfo::<T>::get(&sender).ok_or(Error::<T>::AccountHasNoPet)?;
			ensure!(!PetsInfo::<T>::contains_key(&receiver), Error::<T>::AccountAlreadyHasPet);

			PetsInfo::<T>::insert(&receiver, (id, pet));
			PetsInfo::<T>::remove(&sender);

			Self::deposit_event(Event::PetTransfered(sender, receiver, id));

			Ok(().into())
		}

		/// Feed the pet.
		///
		/// - id: The id of the pet
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn feed(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let (id, _) = PetsInfo::<T>::get(&sender).ok_or(Error::<T>::AccountHasNoPet)?;

			LastFeedTime::<T>::insert(id, frame_system::Pallet::<T>::block_number());

			Self::deposit_event(Event::PetFeeded(sender, id));

			Ok(().into())
		}

		/// Pet is sleep.
		///
		/// - id: The id of the pet
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn sleep(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let (id, _) = PetsInfo::<T>::get(&sender).ok_or(Error::<T>::AccountHasNoPet)?;

			LastSleepTime::<T>::insert(id, frame_system::Pallet::<T>::block_number());

			Self::deposit_event(Event::PetSleeped(sender, id));

			Ok(().into())
		}

	}
}