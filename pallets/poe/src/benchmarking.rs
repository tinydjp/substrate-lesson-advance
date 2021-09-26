//! Benchmarking setup for pallet-poe
// 本示例用于展示第6课的作业，参照作业要求，本示例使用了第一节课的 poe module 来测试 benchmark.
// 由于时间所限，暂时只选取了1个方法，即 create_claim。运行图片位于项目的 docs 目录，json 文件位于根目录
// TODO: 研究 extrinsics 参数如何支持通配符，参考 parity 官方的 tips module 优化 benchmark 的测试用例

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
