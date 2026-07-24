#![allow(unused)]

use enum_info::enum_info;

#[test]
fn test_dimensions() {
	// Should have 3 variants: D1 {x}, D2 {x, y}, D3 {x, y, z}
	#[enum_info]
	#[derive(PartialEq, Clone, Debug, Copy)]
	enum Vector {
		D1 {x: f32},
		D2 {x: f32, y: f32},
		D3 {x: f32, y: f32, z: f32}
	}
	
	assert_eq!(Vector::variant_count(), 3);
}