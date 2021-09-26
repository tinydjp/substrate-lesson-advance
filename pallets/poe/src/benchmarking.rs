//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as POE;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	create_claim {
		let s in 0 .. 100;
		let claim = b"claim".to_vec();
		let caller: T::AccountId = whitelisted_caller();
		//let current_block = <frame_system::Pallet<T>>::block_number();
	}: _(RawOrigin::Signed(caller.clone()), claim.clone())
	verify {
		let current_block = <frame_system::Pallet<T>>::block_number();
		assert_eq!(Proofs::<T>::get(claim), (caller, current_block));
	}
}

impl_benchmark_test_suite!(POE, crate::mock::new_test_ext(), crate::mock::Test);
