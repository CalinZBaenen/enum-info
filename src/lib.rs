// enum-info
// 
// Written by Calin Z. Baenen, 2026/07/19

//! `enum-info` is a crate to generate an enum `impl` which can tell you the
//!  number of – and, in most cases, list the – variants in an enum.

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
use core::num::NonZeroUsize;
use std::vec::Vec;





#[allow(unused)]
trait SubposData<S:Subpos> {
	fn position(&self) -> &S;
	
	fn data(&self) -> &S::Data;
}



trait Subpos:PartialEq {
	type Data:PartialEq;
	
	const ASSOCIATED_STAGE:LexicalPosStage;
}

impl Subpos for () {
	type Data = ();
	
	const ASSOCIATED_STAGE:LexicalPosStage = LexicalPosStage::Start;
}





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

impl Subpos for GenericParamPos {
	type Data = NonZeroUsize;
	
	const ASSOCIATED_STAGE:LexicalPosStage = LexicalPosStage::Generics;
}



#[derive(PartialOrd, PartialEq,Clone, Debug, Copy, Ord, Eq)]
#[repr(usize)]
enum LexicalPosStage {
	/// Indicates anything before, and up to, the `enum` keyword.
	Start = 0,
	/// Indicates the position of the name of the enum.
	Name = 1,
	/// Indicates the position directly following the name but before the generics and enum body.
	AfterName = 2,
	/// Indicates the position between the outermost `<`/`>`.
	/// 
	/// If `depth` is greater than `1`, `pos` should not be [`GenericParamPos::Name`].
	Generics = 3,
	/// Indicates the position directly following the generics.
	AfterGenerics = 4,
	/// Indicates the position after the `where` keyword but before the enum body.
	WhereClause = 5,
	/// Indicates the position inside an enum body.
	EnumBody = 6
}



#[derive(Clone, Copy)]
union LexicalPosData {
	enum_body:(EnumBodyPos, <EnumBodyPos as Subpos>::Data),
	generics:(GenericParamPos, <GenericParamPos as Subpos>::Data),
	usize:usize,
	_marker:()
}

impl SubposData<EnumBodyPos> for LexicalPosData {
	fn position(&self) -> &EnumBodyPos { unsafe { &self.enum_body.0 } }
	
	fn data(&self) -> &<EnumBodyPos as Subpos>::Data { unsafe { &self.enum_body.1 } }
}

impl SubposData<GenericParamPos> for LexicalPosData {
	fn position(&self) -> &GenericParamPos { unsafe { &self.generics.0 } }
	
	fn data(&self) -> &<GenericParamPos as Subpos>::Data { unsafe { &self.generics.1 } }
}



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



#[derive(PartialEq, Clone, Copy, Eq)]
enum EnumBodyPos {
	/// Indicates the position of an enum variant.
	Variant,
	/// Indicates the position after an enum variant's
	///  name but before the `=`.
	AfterVariant,
	/// Indicates the position following the equals
	///  sign in an enum variant declaration.
	Definition
}

impl Subpos for EnumBodyPos {
	type Data = ();
	
	const ASSOCIATED_STAGE:LexicalPosStage = LexicalPosStage::EnumBody;
}



/// The expected lexical position of a token.
#[derive(Clone, Copy)]
struct LexicalPos {
	pub stage:LexicalPosStage,
	pub data:LexicalPosData
}

impl LexicalPos {
	#[inline(always)] pub const fn after_generics() -> Self {
		Self {
			stage: LexicalPosStage::AfterGenerics,
			data: LexicalPosData { _marker: () }
		}
	}
	
	#[inline(always)] pub const fn where_clause() -> Self {
		Self {
			stage: LexicalPosStage::WhereClause,
			data: LexicalPosData { usize: 0 }
		}
	}
	
	#[inline(always)] pub const fn after_name() -> Self {
		Self {
			stage: LexicalPosStage::AfterName,
			data: LexicalPosData { _marker: () }
		}
	}
	
	#[inline(always)] pub const fn enum_body(pos:EnumBodyPos) -> Self {
		Self {
			stage: LexicalPosStage::EnumBody,
			data: LexicalPosData { enum_body: (pos, ()) }
		}
	}
	
	#[inline(always)] pub const fn generics(depth:<GenericParamPos as Subpos>::Data, pos:GenericParamPos) -> Self {
		Self {
			stage: LexicalPosStage::Generics,
			data: LexicalPosData { generics: (pos, depth) }
		}
	}
	
	#[inline(always)] pub const fn start() -> Self {
		Self {
			stage: LexicalPosStage::Start,
			data: LexicalPosData { _marker: () }
		}
	}
	
	#[inline(always)] pub const fn name() -> Self {
		Self {
			stage: LexicalPosStage::Name,
			data: LexicalPosData { _marker: () }
		}
	}
	
	/// Returns whether [`LexicalPosStage::EnumBody`] is a valid next position.
	#[inline(always)] pub const fn enum_body_can_follow(&self) -> bool {
		   self.stage as usize == LexicalPosStage::AfterName       as usize
		|| self.stage as usize == LexicalPosStage::AfterGenerics   as usize
		|| (
		       self.stage as usize == LexicalPosStage::WhereClause as usize
		    && unsafe { self.data.usize } == 0
		)
	}
	
	pub fn after_substate<S>(&self, substate:S) -> bool where LexicalPosData:SubposData<S>, S:Subpos+Ord {
		   self.stage == S::ASSOCIATED_STAGE
		&& <LexicalPosData as SubposData<S>>::position(&self.data).gt(&substate)
	}
	
	/// Returns how deep into generics the lexical position is.
	/// 
	/// If the lexical position is not inside generics, `0` is returned.
	#[inline(always)] pub const fn generics_depth(&self) -> usize {
		if self.stage as usize != LexicalPosStage::Generics as usize { return 0; }
		unsafe {
			self.data.generics.1.get()
		}
	}
	
	pub fn in_substate<S>(&self, substate:S) -> bool where LexicalPosData:SubposData<S>, S:Subpos {
		   self.stage == S::ASSOCIATED_STAGE
		&& <LexicalPosData as SubposData<S>>::position(&self.data).eq(&substate)
	}
}

impl PartialOrd<LexicalPosStage> for LexicalPos {
	#[inline(always)] fn partial_cmp(&self, rhs:&LexicalPosStage) -> Option<Ordering> { Some(self.stage.cmp(rhs)) }
}

impl PartialOrd for LexicalPos {
	/// Compares two [`LexicalPos`]es on the basis of which stage they are in.
	#[inline(always)] fn partial_cmp(&self, rhs:&Self) -> Option<Ordering> { Some(self.cmp(rhs)) }
}

impl PartialEq<LexicalPosStage> for LexicalPos {
	#[inline(always)] fn eq(&self, rhs:&LexicalPosStage) -> bool { self.stage.eq(rhs) }
}

impl PartialEq for LexicalPos {
	#[inline] fn eq(&self, rhs:&Self) -> bool {
		match (self.stage, rhs.stage) {
			(LexicalPosStage::Generics, LexicalPosStage::Generics) => { unsafe {
				self.data.generics == rhs.data.generics
			} }
			
			(LexicalPosStage::EnumBody, LexicalPosStage::EnumBody) => { unsafe {
				self.data.enum_body == rhs.data.enum_body
			} }
			
			(x, y) => { x == y }
		}
	}
}

impl Ord for LexicalPos {
	/// Compares two [`LexicalPos`]es on the basis of which stage they are in.
	#[inline(always)] fn cmp(&self, rhs:&Self) -> Ordering { self.stage.cmp(&rhs.stage) }
}

impl Eq for LexicalPos {}





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
	let mut position = LexicalPos::start();
	let mut output = item.clone();
	let mut name = String::new();
	
	for token in item {
		let mut generic_param_update = false;
		let mut tok_as_punct = None;
		let mut appendage = None;
		
		// Loop over the tokens of the outer item's content.
		match token {
			TokenTree::Group(g) if position.enum_body_can_follow() && !cmp_punct(prev_tok_as_punct.clone(), Some(Punct::new('!', Spacing::Alone))) && g.delimiter() == Delimiter::Brace => {
				position = LexicalPos::enum_body(EnumBodyPos::Variant);
				for subtoken in g.stream() {
					match subtoken {
						TokenTree::Group(_) if position.in_substate(EnumBodyPos::AfterVariant) => {
							if !all_unit { continue; }
							
							variant_names.clear();
							variant_names.shrink_to(0);
							all_unit = false;
						}
						
						TokenTree::Ident(i) if position.in_substate(EnumBodyPos::Variant) => {
							position.data.enum_body.0 = EnumBodyPos::AfterVariant;
							variant_ct += 1;
							
							if all_unit { variant_names.push(i); }
						}
						
						TokenTree::Punct(p) => {
							match p.as_char() {
								'=' => {
									position.data.enum_body.0 = EnumBodyPos::Definition;
								}
								
								',' => {
									position.data.enum_body.0 = EnumBodyPos::Variant;
								}
								
								_ => {}
							}
						}
						
						_ => {}
					}
				}
			}
			
			TokenTree::Group(g) if (position.after_substate(GenericParamPos::Name) || position == LexicalPosStage::WhereClause) => {
				appendage = Some(TokenTree::Group(g));
			}
			
			TokenTree::Ident(i) if position == LexicalPosStage::Name => {
				name = i.to_string();
				position = LexicalPos::after_name();
				continue;
			}
			
			TokenTree::Ident(i) => {
				match i.to_string().as_str() {
					"enum" if position == LexicalPosStage::Start => {
						position = LexicalPos::name();
					},
					
					// Const generics.
					"const" if position.generics_depth() == 1
					        && current_generic_param.name.is_none()
					        && current_generic_param.param_type == Type => { current_generic_param.param_type = Const; },
					
					// Where clause.
					"where" if position == LexicalPosStage::AfterName || position == LexicalPosStage::AfterGenerics => {
						position = LexicalPos::where_clause();
					}
					
					_ if position.in_substate(GenericParamPos::Name) && current_generic_param.name.is_none() => {
						current_generic_param.name = Some(i);
					},
					
					_ if (position.after_substate(GenericParamPos::Name) || position == LexicalPosStage::WhereClause) => {
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
						if position == LexicalPosStage::AfterName {
							position = LexicalPos::generics(unsafe { NonZeroUsize::new_unchecked(1) }, GenericParamPos::Name);
							continue;
						} else if position.after_substate(GenericParamPos::Name) {
							appendage = Some(TokenTree::Punct(p));
							position.data.generics.1 = unsafe { position.data.generics }.1.saturating_add(1);
						} else if position == LexicalPosStage::WhereClause {
							unsafe { position.data.usize += 1; }
							appendage = Some(TokenTree::Punct(p));
						}
					}
					
					// Close generics.
					'>' if position == LexicalPosStage::Generics && !cmp_punct(prev_tok_as_punct.clone(), Some(Punct::new('-', Spacing::Joint))) => 'close_generics: {
						if unsafe { position.data.generics }.1.get() == 1 {
							generic_param_update = true;
							position = LexicalPos::after_generics();
							
							break 'close_generics;
						}
						
						position.data.generics.1 =
							unsafe { NonZeroUsize::new_unchecked(position.data.generics.1.get().saturating_sub(1)) };
						appendage = Some(TokenTree::Punct(p));
					}
					
					// Close generics (in `where` clauses).
					'>' if position == LexicalPosStage::WhereClause && !cmp_punct(prev_tok_as_punct.clone(), Some(Punct::new('-', Spacing::Joint))) => 'close_generics: { unsafe {
						appendage = Some(TokenTree::Punct(p));
						
						if position.data.usize < 1 { break 'close_generics; }
						position.data.usize -= 1;
					} }
					
					// Move from the name part of generics to the bounds.
					':' if position.in_substate(GenericParamPos::Name) => {
						position.data.generics.0 = GenericParamPos::Bounds;
					}
					
					// Capture default values for generic parameters.
					'=' if !position.after_substate(GenericParamPos::Bounds) && position == LexicalPosStage::Generics => {
						position.data.generics.0 = GenericParamPos::DefaultAssignment;
					}
					
					// Push the current generic parameter.
					',' if position.generics_depth() == 1 => {
						position.data.generics.0 = GenericParamPos::Name;
						generic_param_update = true;
					}
					
					_ if (position.after_substate(GenericParamPos::Name) || position == LexicalPosStage::WhereClause) => {
						appendage = Some(TokenTree::Punct(p));
					}
					
					_ => {}
				}
			}
			
			TokenTree::Literal(l) if (position.after_substate(GenericParamPos::Name) || position == LexicalPosStage::WhereClause) => {
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
				} else if position == LexicalPosStage::WhereClause {
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
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("core", Span::call_site())),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("primitive", Span::call_site())),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
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
				TokenTree::Punct(Punct::new(':', Spacing::Joint)),
				TokenTree::Punct(Punct::new(':', Spacing::Alone)),
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
		
		for generic_item in generic_params {
			if generic_item.param_type == Lifetime {
				generics_names.extend([ unsafe { generic_item.param_type.prefix().unwrap_unchecked() } ]);
			}
			
			generics_names.extend([
				TokenTree::Ident(unsafe { generic_item.name.clone().unwrap_unchecked() }),
				TokenTree::Punct(Punct::new(',', Spacing::Alone))
			]);
			
			if let Some(prefix) = generic_item.param_type.prefix() {
				generics_full.extend([prefix]);
			}
			
			generics_full.extend([TokenTree::Ident(unsafe { generic_item.name.unwrap_unchecked() })]);
			
			if let Some(suffix) = generic_item.bounds {
				generics_full.extend([
					TokenTree::Punct(Punct::new(':', Spacing::Alone)),
					TokenTree::Group(Group::new(Delimiter::None, suffix))
				]);
			}
			
			generics_full.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
		}
		
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