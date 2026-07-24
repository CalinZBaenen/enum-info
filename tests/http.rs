#![allow(unused)]

use enum_info::enum_info;

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