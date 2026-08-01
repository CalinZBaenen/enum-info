#![allow(unused)]

use enum_info::enum_info;

const FOO:i32 = 50;
const BAR:i32 = 100;

trait Trait<const N:i32> {}

#[test]
fn test_ambiguity() {
	// Should have 1 variant.
	#[enum_info]
	pub enum Ambiguous<T> where T:Trait<{FOO + BAR}> {
		Item(T)
	}
}