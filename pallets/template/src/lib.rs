#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_runtime::{
    offchain::{
       // storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
	   storage::StorageValueRef,
    },
    //traits::Zero,
	sp_std::{str,}
};


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	//use frame_support::Deserialize;
	//use frame_support::traits::Randomness;
	use sp_io::offchain_index;



	#[derive(Debug, Encode, Decode, Default)]
	struct IndexingData(Vec<u8>, u64);
	
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		//type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn extrinsic(origin: OriginFor<T>, number: u64) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			log::info!("extrinsic ==> block_number is {:?}",frame_system::Module::<T>::block_number());
			let key = Self::derive_key(frame_system::Module::<T>::block_number());
			log::info!("extrinsic ==> key is {:?}",&key);

			//#[cfg(feature = "std")]
			//let hex_string:Vec<_> = key.iter().map(|byte| format!("{:02X}", byte)).collect();
			//let hex_key = "".join(format(byte,"02x") for byte in byte_array);
			//log::info!("extrinsic ==> hex string of key is {:?}",hex_string);
			let data: IndexingData = IndexingData(b"submit_number_unsigned".to_vec(), number);
			
			offchain_index::set(&key, &data.encode());
			
			Ok(())
		}


	}

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
			let key = Self::derive_key(block_number);
			log::info!("offchain_worker ==> block_number is : {:?}", block_number);
			log::info!("offchain_worker ==> key is {:?}",key);
			let val_ref = StorageValueRef::persistent(&key);
			log::info!("offchain_worker ==> data is {:?}",val_ref.get::<IndexingData>());
			if let Ok(Some(value)) = val_ref.get::<IndexingData>() {
				log::info!("offchain_worker ==> local storage data: {:?},{:?}",
				str::from_utf8(&value.0).unwrap_or("error"),value.1);
			} else {
				log::info!("offchain_worker ==> Error reading from local storage.");
			}
          	log::info!("offchain_worker ==> Leave from Terry_node_temp_ocw!: {:?}", block_number);
        }
    }

    impl<T: Config> Pallet<T> {
        #[deny(clippy::clone_double_ref)]
        fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
            block_number.using_encoded(|encoded_bn| {
                b"node-template::storage::"
                    .iter()
                    .chain(encoded_bn)
                    .copied()
                    .collect::<Vec<u8>>()
            })
        }


    }


	
}
