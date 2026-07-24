#![allow(unused)]

use enum_info::enum_info;

macro_rules! ty {
	() => { usize }
}

#[test]
fn test_ambiguity() {
	// Should have 0 variants.
	#[enum_info]
	pub enum Ambiguous where ty!{}: Copy {}
}