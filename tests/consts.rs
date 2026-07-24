#![allow(unused)]

use enum_info::enum_info;

const FOO:i32 = 50;
const BAR:i32 = 100;

#[test]
fn test_consts() {
	// Should have 3 variants: Foo, Bar, Baz.
	#[enum_info]
	#[derive(PartialEq, Debug, Eq)]
	#[repr(i32)]
	enum Consts {
		Foo = FOO,
		Bar = BAR,
		Baz = (FOO + BAR)
	}
	
	assert_eq!(Consts::variant_count(), 3);
	assert_eq!(Consts::VARIANTS.len(), 3);
}