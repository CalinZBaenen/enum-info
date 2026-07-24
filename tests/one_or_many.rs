#![allow(unused)]

use enum_info::enum_info;

use core::mem::size_of;

#[test]
fn test_one_or_many() {
	// Should have 2 variants: Many([T; N]), One(T)
	#[enum_info]
	enum OneOrMany<T, const N:usize> {
		Many([T; N]),
		One(T)
	}
	
	assert_eq!(OneOrMany::<(), 0>::variant_count(), 2);
}