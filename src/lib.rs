// enum-info
// 
// Written by Calin Z. Baenen, 2026/07/19
//               last updated: 2026/07/31

//! `enum-info` is a crate to generate an enum `impl` which can tell you the
//!  number of – and, in most cases, list the – variants in an enum.
//! 
//! # Example
//! ```rust
//! use enum_info::enum_info;
//! 
//! #[enum_info]
//! #[derive(PartialEq, Debug, Eq)]
//! enum Characters {
//! 	Katty,
//! 	Jax
//! }
//! 
//! assert_eq!(Characters::variant_count(), 2);
//! assert_eq!(Characters::VARIANTS, [Characters::Katty, Characters::Jax]);
//! ```

use proc_macro::{
	TokenStream,
	Delimiter,
	TokenTree,
	Literal,
	Spacing,
	Group,
	Ident,
	Punct,
	Span
};

use core::cmp::{PartialOrd, PartialEq, Ordering, Ord, Eq};
use std::vec::Vec;





trait HasSubpos<S:Subpos> {
	/// The position of the sub-position.
	/// 
	/// [`::core::option::Option::None`] should be returned if
	///  the associated super-position is not correct.
	fn subpos_position(&self) -> Option<&S>;
}



trait Subpos:Ord+Eq {}





#[derive(PartialEq, Clone, Copy, Eq)]
#[repr(u8)]
enum GenericParamType {
	Type     = 0,
	Const    = 1,
	Lifetime = 2
}

impl GenericParamType {
	#[inline(always)] pub fn prefix(&self) -> Option<TokenTree> {
		match self {
			Self::Type => { None }
			
			Self::Const => { Some( TokenTree::Ident(Ident::new("const", Span::call_site())) ) }
			
			Self::Lifetime => { Some( TokenTree::Punct(Punct::new('\'', Spacing::Joint)) ) }
		}
	}
}

use GenericParamType::*;



/// The position inside of a generic parameter.
#[derive(PartialOrd, PartialEq, Clone, Copy, Ord, Eq)]
enum GenericParamPos {
	Name              = 0,
	Bounds            = 1,
	DefaultAssignment = 2
}

impl Subpos for GenericParamPos {}



#[derive(Clone)]
struct GenericParam {
	pub default_item:Option<TokenStream>,
	pub param_type:GenericParamType,
	pub bounds:Option<TokenStream>,
	pub name:Option<Ident>
}

impl GenericParam {
	pub const DEFAULT:Self = Self {default_item: None, param_type: GenericParamType::Type, bounds: None, name: None};
}



#[derive(PartialOrd, PartialEq, Clone, Copy, Ord, Eq)]
enum EnumBodyPos {
	/// Indicates the position of an enum variant.
	Variant      = 0,
	/// Indicates the position after an enum variant's
	///  name but before the `=`.
	AfterVariant = 1,
	/// Indicates the position following the equals
	///  sign in an enum variant declaration.
	Definition   = 2
}

impl Subpos for EnumBodyPos {}



/// The expected lexical position of a token.
#[derive(PartialEq, Clone, Copy, Eq)]
#[repr(usize)]
enum LexicalPos {
	/// Indicates anything before, and up to, the `enum` keyword.
	Start,
	/// Indicates the position of the name of the enum.
	Name,
	/// Indicates the position directly following the name but before the generics and enum body.
	AfterName,
	/// Indicates the position between the outermost `<`/`>`.
	/// 
	/// [`GenericParamPos::Name`] should not be paired with a nesting level greater than one.
	Generics(GenericParamPos),
	/// Indicates the position directly following the generics.
	AfterGenerics,
	/// Indicates the position after the `where` keyword but before the enum body.
	WhereClause,
	/// Indicates the position inside an enum body.
	EnumBody(EnumBodyPos)
}

impl LexicalPos {
	/// Returns whether [`LexicalPos::EnumBody`] is a valid next position.
	#[inline] pub fn enum_body_can_follow(&self) -> bool {
		   self.eq(&Self::AfterName)
		|| self.eq(&Self::AfterGenerics)
		|| self.eq(&Self::WhereClause)
	}
	
	pub fn after_substate<S>(&self, substate:S) -> bool where Self:HasSubpos<S>, S:Subpos {
		<Self as HasSubpos<S>>::subpos_position(self).is_some_and(|sp| sp.gt(&substate))
	}
	
	pub fn in_substate<S>(&self, substate:S) -> bool where Self:HasSubpos<S>, S:Subpos {
		<Self as HasSubpos<S>>::subpos_position(self).is_some_and(|sp| sp.eq(&substate))
	}
}

impl PartialOrd for LexicalPos {
	/// Compares two [`LexicalPos`]es on the basis of which stage they are in.
	#[inline(always)] fn partial_cmp(&self, rhs:&Self) -> Option<Ordering> { Some(self.cmp(rhs)) }
}

impl HasSubpos<GenericParamPos> for LexicalPos {
	#[inline] fn subpos_position(&self) -> Option<&GenericParamPos> {
		match self {
			Self::Generics(pos) => Some(pos),
			_ => None
		}
	}
}

impl HasSubpos<EnumBodyPos> for LexicalPos {
	#[inline] fn subpos_position(&self) -> Option<&EnumBodyPos> {
		match self {
			Self::EnumBody(pos) => Some(pos),
			_ => None
		}
	}
}

impl Ord for LexicalPos {
	/// Compares two [`LexicalPos`]es on the basis of which stage they are in.
	#[inline(always)] fn cmp(&self, rhs:&Self) -> Ordering {
		unsafe { (*(self as *const Self as *const usize)).cmp((rhs as *const Self as *const usize).as_ref_unchecked()) }
	}
}





fn double_colon() -> TokenTree {
	TokenTree::Group(Group::new(Delimiter::None, TokenStream::from_iter( [
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone))
	].into_iter() )))
}



/// Checks if two [`Option<Punct>`]s are equal based on their [`char`] and
///  [`Spacing`] values.
/// 
/// If either value is [`Option::None`], then `false` is returned, regardless
///  of whether both inputs are [`Option::None`].
#[inline] fn cmp_punct(punct_a:Option<Punct>, punct_b:Option<Punct>) -> bool {
	let (Some(a), Some(b)) = (punct_a, punct_b) else { return false; };
	
	   a.as_char() == b.as_char()
	&& a.spacing() == b.spacing()
}



/// Generates an item `impl` with a guaranteed `variant_count` method and an
///  associated constant, `VARIANTS`, for enums whose variants are all unit-like.
/// 
/// `variant_count` is a constant function which returns the number of variants
///  the enum has.
/// 
/// `VARIANTS` is an array whose elements consist of each enum variant in order.
#[proc_macro_attribute]
pub fn enum_info(_attr:TokenStream, item:TokenStream) -> TokenStream {
	let mut current_generic_param = GenericParam::DEFAULT;
	let mut prev_tok_as_punct = None;
	let mut generic_params = Vec::<GenericParam>::new();
	let mut variant_names = Vec::<Ident>::new();
	let mut where_clause = None;
	let mut variant_ct = 0;
	let mut all_unit = true;
	let mut position = LexicalPos::Start;
	let mut nesting = 0usize;
	let mut output = item.clone();
	let mut name = String::new();
	
	for token in item {
		let mut generic_param_update = false;
		let mut tok_as_punct = None;
		let mut appendage = None;
		
		// Loop over the tokens of the outer item's content.
		match token {
			TokenTree::Group(g) if position.enum_body_can_follow() && !cmp_punct(prev_tok_as_punct.clone(), Some(Punct::new('!', Spacing::Alone))) && g.delimiter() == Delimiter::Brace => {
				position = LexicalPos::EnumBody(EnumBodyPos::Variant);
				for subtoken in g.stream() {
					match subtoken {
						TokenTree::Group(_) if position.in_substate(EnumBodyPos::AfterVariant) => {
							if !all_unit { continue; }
							
							variant_names.clear();
							variant_names.shrink_to(0);
							all_unit = false;
						}
						
						TokenTree::Ident(i) if position.in_substate(EnumBodyPos::Variant) => {
							position = LexicalPos::EnumBody(EnumBodyPos::AfterVariant);
							variant_ct += 1;
							
							if all_unit { variant_names.push(i); }
						}
						
						TokenTree::Punct(p) => {
							match p.as_char() {
								'=' => {
									position = LexicalPos::EnumBody(EnumBodyPos::Definition);
								}
								
								',' => {
									position = LexicalPos::EnumBody(EnumBodyPos::Variant);
								}
								
								_ => {}
							}
						}
						
						_ => {}
					}
				}
			}
			
			TokenTree::Group(g) if (position.after_substate(GenericParamPos::Name) || position == LexicalPos::WhereClause) => {
				appendage = Some(TokenTree::Group(g));
			}
			
			TokenTree::Ident(i) if position == LexicalPos::Name => {
				name = i.to_string();
				position = LexicalPos::AfterName;
				continue;
			}
			
			TokenTree::Ident(i) => {
				match i.to_string().as_str() {
					"enum" if position == LexicalPos::Start => {
						position = LexicalPos::Name;
					},
					
					// Const generics.
					"const" if nesting == 0
					        && current_generic_param.name.is_none()
					        && current_generic_param.param_type == Type => { current_generic_param.param_type = Const; },
					
					// Where clause.
					"where" if position == LexicalPos::AfterName || position == LexicalPos::AfterGenerics => {
						position = LexicalPos::WhereClause;
					}
					
					_ if position.in_substate(GenericParamPos::Name) && current_generic_param.name.is_none() => {
						current_generic_param.name = Some(i);
					},
					
					_ if (position.after_substate(GenericParamPos::Name) || position == LexicalPos::WhereClause) => {
						appendage = Some(TokenTree::Ident(i));
					}
					
					_ => {}
				}
			}
			
			TokenTree::Punct(p) => {
				let c = p.as_char();
				tok_as_punct = Some(p.clone());
				
				match c {
					// Lifetimes.
					'\'' if position.in_substate(GenericParamPos::Name)
					     && current_generic_param.name.is_none()
					     && current_generic_param.param_type == Type => { current_generic_param.param_type = Lifetime; }
					
					// Open generics.
					'<' => {
						if position == LexicalPos::AfterName {
							position = LexicalPos::Generics(GenericParamPos::Name);
							nesting = 0;
							continue;
						} else if position.after_substate(GenericParamPos::Name) {
							appendage = Some(TokenTree::Punct(p));
							nesting = nesting.saturating_add(1);
						} else if position == LexicalPos::WhereClause {
							nesting = nesting.wrapping_add(1);
							appendage = Some(TokenTree::Punct(p));
						}
					}
					
					// Close generics.
					'>' if matches!(position, LexicalPos::Generics(_)) && !cmp_punct(prev_tok_as_punct.clone(), Some(Punct::new('-', Spacing::Joint))) => 'close_generics: {
						if nesting == 0 {
							generic_param_update = true;
							position = LexicalPos::AfterGenerics;
							
							break 'close_generics;
						}
						
						nesting = nesting.saturating_sub(1);
						appendage = Some(TokenTree::Punct(p));
					}
					
					// Close generics (in `where` clauses).
					'>' if position == LexicalPos::WhereClause && !cmp_punct(prev_tok_as_punct.clone(), Some(Punct::new('-', Spacing::Joint))) => {
						appendage = Some(TokenTree::Punct(p));
						nesting = nesting.wrapping_sub(1);
					}
					
					// Move from the name part of generics to the bounds.
					':' if position.in_substate(GenericParamPos::Name) => {
						position = LexicalPos::Generics(GenericParamPos::Bounds);
					}
					
					// Capture default values for generic parameters.
					'=' if !position.after_substate(GenericParamPos::Bounds) && matches!(position, LexicalPos::Generics(_)) => {
						position = LexicalPos::Generics(GenericParamPos::DefaultAssignment);
					}
					
					// Push the current generic parameter.
					',' if matches!(position, LexicalPos::Generics(_)) && nesting == 0 => {
						position = LexicalPos::Generics(GenericParamPos::Name);
						generic_param_update = true;
					}
					
					_ if (position.after_substate(GenericParamPos::Name) || position == LexicalPos::WhereClause) => {
						appendage = Some(TokenTree::Punct(p));
					}
					
					_ => {}
				}
			}
			
			TokenTree::Literal(l) if (position.after_substate(GenericParamPos::Name) || position == LexicalPos::WhereClause) => {
				appendage = Some(TokenTree::Literal(l));
			}
			
			_ => {}
		}
		
		'append_generics: {
			if let Some(appendage) = appendage {
				let append_to:&mut Option<TokenStream>;
				if position.in_substate(GenericParamPos::Bounds) {
					append_to = &mut current_generic_param.bounds;
				} else if position.in_substate(GenericParamPos::DefaultAssignment) {
					append_to = &mut current_generic_param.default_item;
				} else if position == LexicalPos::WhereClause {
					append_to = &mut where_clause;
				} else {
					break 'append_generics;
				}
				
				if let &mut Some(ref mut append_to) = append_to {
					append_to.extend([appendage].into_iter());
				} else {
					*append_to = Some(TokenStream::from_iter([appendage].into_iter()));
				}
			}
		}
		
		'add_generic_param: {
			if current_generic_param.name.is_none() { break 'add_generic_param; }
			
			if generic_param_update {
				generic_params.push(current_generic_param);
				current_generic_param = GenericParam::DEFAULT;
			}
		}
		
		prev_tok_as_punct = tok_as_punct;
	}
	
	let mut r#impl = TokenStream::from_iter([
		// #[inline(always)]
		TokenTree::Punct(Punct::new('#', Spacing::Alone)),
		TokenTree::Group(Group::new(
			Delimiter::Bracket,
			TokenStream::from_iter([
				TokenTree::Ident(Ident::new("inline", Span::call_site())),
				TokenTree::Group(Group::new(
					Delimiter::Parenthesis,
					TokenStream::from_iter([
						TokenTree::Ident(Ident::new("always", Span::call_site()))
					].into_iter())
				))
			].into_iter())
		)),
		
		// /// ...
		// pub const fn variant_count() -> ::core::primitive::usize
		
		TokenTree::Punct(Punct::new('#', Spacing::Alone)),
		TokenTree::Group(Group::new(
			Delimiter::Bracket,
			TokenStream::from_iter([
				TokenTree::Ident(Ident::new("doc", Span::call_site())),
				TokenTree::Punct(Punct::new('=', Spacing::Alone)),
				TokenTree::Literal(Literal::string("Returns the number of variants this enum has."))
			].into_iter())
		)),
		TokenTree::Ident(Ident::new("pub", Span::call_site())),
		TokenTree::Ident(Ident::new("const", Span::call_site())),
		TokenTree::Ident(Ident::new("fn", Span::call_site())),
		TokenTree::Ident(Ident::new("variant_count", Span::call_site())),
		TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
		TokenTree::Punct(Punct::new('-', Spacing::Joint)),
		TokenTree::Punct(Punct::new('>', Spacing::Alone)),
		double_colon(),
		TokenTree::Ident(Ident::new("core", Span::call_site())),
		double_colon(),
		TokenTree::Ident(Ident::new("primitive", Span::call_site())),
		double_colon(),
		TokenTree::Ident(Ident::new("usize", Span::call_site())),
		
		// { $variant_ct }
		TokenTree::Group(Group::new(
			Delimiter::Brace,
			TokenStream::from_iter([
				TokenTree::Literal(Literal::usize_unsuffixed(variant_ct))
			].into_iter())
		))
	].into_iter());
	
	// pub const VARIANTS:[Self; $variant_ct] = [...$variant_names];
	if all_unit {
		let mut items = TokenStream::new();
		
		// [...$variant_names]
		for variant in variant_names {
			items.extend([
				TokenTree::Ident(Ident::new("Self", Span::call_site())),
				double_colon(),
				TokenTree::Ident(variant),
				TokenTree::Punct(Punct::new(',', Spacing::Alone))
			])
		}
		
		r#impl.extend([
			TokenTree::Ident(Ident::new("pub", Span::call_site())),
			TokenTree::Ident(Ident::new("const", Span::call_site())),
			TokenTree::Ident(Ident::new("VARIANTS", Span::call_site())),
			TokenTree::Punct(Punct::new(':', Spacing::Alone)),
			TokenTree::Group(Group::new(
				Delimiter::Bracket,
				TokenStream::from_iter([
					TokenTree::Ident(Ident::new("Self", Span::call_site())),
					TokenTree::Punct(Punct::new(';', Spacing::Alone)),
					TokenTree::Literal(Literal::usize_unsuffixed(variant_ct))
				].into_iter())
			)),
			TokenTree::Punct(Punct::new('=', Spacing::Alone)),
			TokenTree::Group(Group::new(Delimiter::Bracket, items)),
			TokenTree::Punct(Punct::new(';', Spacing::Alone))
		]);
	}
	
	let mut generics_names = TokenStream::new();
	let mut generics_full = TokenStream::new();
	
	if generic_params.len() > 0 {
		generics_names.extend([TokenTree::Punct(Punct::new('<', Spacing::Alone))]);
		generics_full.extend([TokenTree::Punct(Punct::new('<', Spacing::Alone))]);
		
		for generic_item in generic_params { unsafe {
			if generic_item.param_type == Lifetime {
				generics_names.extend([ generic_item.param_type.prefix().unwrap_unchecked() ]);
			}
			
			generics_names.extend([
				TokenTree::Ident( generic_item.name.clone().unwrap_unchecked() ),
				TokenTree::Punct(Punct::new(',', Spacing::Alone))
			]);
			
			if let Some(prefix) = generic_item.param_type.prefix() {
				generics_full.extend([prefix]);
			}
			
			generics_full.extend([TokenTree::Ident( generic_item.name.unwrap_unchecked() )]);
			
			if let Some(suffix) = generic_item.bounds {
				generics_full.extend([
					TokenTree::Punct(Punct::new(':', Spacing::Alone)),
					TokenTree::Group(Group::new(Delimiter::None, suffix))
				]);
			}
			
			generics_full.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
		} }
		
		generics_names.extend([TokenTree::Punct(Punct::new('>', Spacing::Alone))]);
		generics_full.extend([TokenTree::Punct(Punct::new('>', Spacing::Alone))]);
	}
	
	output.extend([
		// impl<...$generic_parameters> $name<...$generic_parameter_names>
		TokenTree::Ident(Ident::new("impl", Span::call_site())),
		TokenTree::Group(Group::new(Delimiter::None, generics_full)),
		TokenTree::Ident(Ident::new(&name, Span::call_site())),
		TokenTree::Group(Group::new(Delimiter::None, generics_names)),
		
		// where $where_clause
		TokenTree::Group(
			Group::new(
				Delimiter::None,
				where_clause.map(
					|ts| {
						let mut w = TokenStream::from_iter([TokenTree::Ident(Ident::new("where", Span::call_site()))]);
						w.extend(ts);
						w
					}
				).unwrap_or_default()
			)
		),
		
		// { /* ... */ }
		TokenTree::Group(Group::new(Delimiter::Brace, r#impl))
	]);
	output
}