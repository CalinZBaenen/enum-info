#![allow(unused)]

use enum_info::enum_info;

#[allow(deprecated)]
#[test]
fn test_attributes() {
	// Should have 3 variants: Stylish, Zenful, Outdated
	// Should receive `VARIANTS` associated constant.
	#[enum_info]
	#[derive(PartialEq, Debug, Eq)]
	enum Trendiness {
		Stylish,
		Zenful,
		
		#[deprecated]
		Outdated
	}
	
	assert_eq!(Trendiness::variant_count(), 3);
	assert_eq!(Trendiness::VARIANTS[2], Trendiness::Outdated);
}