#![allow(unused)]

use enum_info::enum_info;

#[test]
fn test_racers() {
	// Should have 8 variants.
	#[enum_info]
	enum KartRacers {
		Katty,
		Jax,
		Jo,
		Mayo,
		Guest1,
		Guest2,
		Guest3,
		Guest4
	}
	
	assert_eq!(KartRacers::variant_count(), 8);
}

#[test]
fn test_http() {
	// Should have 27 variants.
	#[derive(PartialOrd, PartialEq, Clone, Copy, Ord, Eq)]
	#[repr(u16)]
	#[enum_info]
	enum HttpStatus {
		Continue = 100,
		SwitchingProtocols,
		Processing,
		EarlyHints,
		
		OK = 200,
		Created,
		Accepted,
		NonAuthoritativeInformation,
		NoContent,
		ResetContent,
		PartialContent,
		
		MultipleChoices = 300,
		Moved,
		Found,
		SeeOther,
		NotModified,
		TemporaryRedirect = 307,
		PermanentRedirect,
		
		BadRequest = 400,
		Unauthorized,
		PaymentRequired,
		Forbidden,
		NotFound,
		MethodNotAllowed,
		NotAcceptable,
		ProxyAuthenticationRequired,
		RequestTimeout
	}
	
	assert_eq!(HttpStatus::variant_count(), 27);
	assert_eq!(HttpStatus::VARIANTS[22] as u16, 404);
}

#[test]
fn test_consts() {
	const FOO:isize = 50;
	const BAR:isize = 100;
	
	// Should have 3 variants: Foo, Bar, Baz.
	#[enum_info]
	#[derive(PartialEq, Debug, Eq)]
	enum Consts {
		Foo = FOO,
		Bar = BAR,
		Baz = isize::MIN
	}
	
	assert_eq!(Consts::variant_count(), 3);
}

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

#[test]
fn test_ambiguity() {
	macro_rules! ty {
		() => { usize }
	}
	
	// Should have 0 variants.
	#[enum_info]
	pub enum Ambiguous where ty!{}: Copy {}
}

#[test]
fn test_fn_arrow() {
	use core::marker::PhantomData;
	use core::iter::IntoIterator;
	use core::ops::FnOnce;
	
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