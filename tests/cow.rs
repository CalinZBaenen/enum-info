#![allow(unused)]

use enum_info::enum_info;

#[test]
fn test_cow() {
	// Should have 2 variants: Borrowed(B), Owned(B::ToOwned)
	#[enum_info]
	pub enum Cow<'a, B: ?Sized + 'a> where B: ToOwned {
		Borrowed(&'a B),
		Owned(<B as ToOwned>::Owned),
	}
	
	assert_eq!(Cow::<()>::variant_count(), 2);
}