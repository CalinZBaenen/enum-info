#![allow(unused)]

use enum_info::enum_info;

use core::marker::PhantomData;
use core::iter::IntoIterator;
use core::ops::FnOnce;

#[test]
fn test_fn_arrow() {
	type TestFnType = fn()->usize;
	
	#[enum_info]
	enum ContainsFnGeneric<F:FnOnce() -> usize> {
		DummyItem(PhantomData<F>)
	}
	
	#[enum_info]
	enum ContainsFnWhere<F> where Vec<F>: IntoIterator<Item=fn()->usize> {
		DummyItem(PhantomData<F>)
	}
	
	assert_eq!(ContainsFnGeneric::<TestFnType>::variant_count(), 1);
	assert_eq!(ContainsFnWhere::<TestFnType>::variant_count(), 1);
}