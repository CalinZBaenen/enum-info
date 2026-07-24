#![allow(unused)]

use enum_info::enum_info;

#[test]
fn test_option() {
	// Should have 2 variants: Some(T), None
	#[enum_info]
	enum Optional<T> {
		Some(T),
		None
	}
	
	assert_eq!(Optional::<()>::variant_count(), 2);
}