#![allow(unused)]

use enum_info::enum_info;

const FOO:isize = 50;
const BAR:isize = 100;

#[test]
fn test_doc_comments() {
	// Should have two variants: Documented, Undocumented
	// Should receive `VARIANTS` associated constant.
	#[enum_info]
	#[derive(PartialEq, Debug, Eq)]
	enum Documentation {
		/// This variant is well-documented.
		Documented,
		Undocumented
	}
	
	assert_eq!(Documentation::variant_count(), 2);
	assert_eq!(Documentation::VARIANTS[0], Documentation::Documented);
}