use
{
	std::collections::HashMap,
	proc_macro::TokenStream,

	heck::CamelCase,
	quote::{format_ident, quote},
	syn::{AttributeArgs, ItemFn, Lit, Meta, NestedMeta, Path, punctuated::Punctuated, ReturnType, token::Paren, Type::Tuple, TypeTuple},
};

/// # Summary
///
/// This macro takes some `fn` and generates a unique [error](std::error::Error) for it to return.
///
/// # Parameters
///
/// All [errors](std::error::Error)s which may be returned by the function should be passed to the macro,
/// separated by commas:
///
/// ```ignore
/// #[errors(bincode::Error, std::io::Error)]
/// ```
///
/// > ## Note
/// >
/// > This macro uses the paths to each `Error` to determine the name of the generated variants.
/// > That is to say `std::io::Error` will map to `StdIo`, and `io::Error` will map to `Io`.
///
/// You can override the default naming by assigning the `Error` an alias:
///
/// ```ignore
/// #[errors(bincode::Error, std::io::Error = "IoError")]
/// ```
///
/// # Remarks
///
/// The visibility of the generated `Error` will be the same as the function which it is attached
/// to. If the function is `pub(crate)`, the `Error` will be `pub(crate)` also.
///
/// The name of the function also determines the name of the generated `Error`. See the example.
///
/// # Example
///
/// ```
/// use std::{fmt, fs, path::Path};
/// use erro_rs::errors;
///
/// match read_int("/tmp/foo") {
///     Ok(i) => println!("Was an `Ok`: {}", i),
///     Err(ReadIntError::StdIo(e)) => println!("`io::Error`: {}", e),
///     Err(ReadIntError::StdNumParseInt(e)) => println!("`ParseIntError`: {}", e),
/// };
///
/// #[errors(std::io::Error, std::num::ParseIntError)]
/// fn read_int(path: impl AsRef<Path>) -> i128 {
///     let content = fs::read_to_string(path)?;
///     let number = content.parse::<i128>()?;
///     Ok(number)
/// }
/// ```
///
/// The above is equivalent to the following:
///
/// ```
/// use std::{error::Error, fmt, fs, io, num::ParseIntError, path::Path};
///
/// match read_int("/tmp/foo") {
///     Ok(i) => println!("Was an `Ok`: {}", i),
///     Err(ReadIntError::StdIo(e)) => println!("`io::Error`: {}", e),
///     Err(ReadIntError::StdNumParseInt(e)) => println!("`ParseIntError`: {}", e),
/// };
///
/// fn read_int(path: impl AsRef<Path>) -> Result<i128, ReadIntError> {
///     let content = fs::read_to_string(path)?;
///     let number = content.parse::<i128>()?;
///     Ok(number)
/// }
///
/// #[derive(Debug)]
/// enum ReadIntError {
///     StdIo(io::Error),
///     StdNumParseInt(ParseIntError),
/// }
///
/// impl fmt::Display for ReadIntError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             Self::StdIo(e) => write!(f, "{}", e),
///             Self::StdNumParseInt(e) => write!(f, "{}", e),
///         }
///     }
/// }
///
/// impl From<io::Error> for ReadIntError {
///     fn from(e: io::Error) -> Self {
///         Self::StdIo(e)
///     }
/// }
///
/// impl From<ParseIntError> for ReadIntError {
///     fn from(e: ParseIntError) -> Self {
///         Self::StdNumParseInt(e)
///     }
/// }
///
/// impl Error for ReadIntError {}
/// ```
#[proc_macro_attribute]
pub fn errors(attr: TokenStream, item: TokenStream) -> TokenStream
{
	let attr_args = syn::parse_macro_input!(attr as AttributeArgs);
	let attr_args_parsed: HashMap<_, _> = attr_args.into_iter().filter_map(|a| match a
	{
		NestedMeta::Meta(Meta::Path(p)) => Some((p, None)),
		NestedMeta::Meta(Meta::NameValue(v)) => Some((v.path, Some(v.lit))),
		_ => None,
	}).collect();

	if let Ok(function) = syn::parse::<ItemFn>(item.clone())
	{
		return parse_fn(function, attr_args_parsed);
	}

	panic!("The #[errors] macro can only be used on functions");
}

fn parse_fn(function: ItemFn, errors: HashMap<Path, Option<Lit>>) -> TokenStream
{
	let attrs = function.attrs;
	let block = function.block;
	let vis = function.vis;

	let constness = function.sig.constness;
	let asyncness = function.sig.asyncness;
	let unsafety = function.sig.unsafety;
	let abi = function.sig.abi;
	let fn_token = function.sig.fn_token;
	let ident = function.sig.ident;
	let generics = function.sig.generics.clone();
	let where_clause = function.sig.generics.where_clause;
	let inputs = function.sig.inputs.into_iter();
	let variadic = function.sig.variadic;
	let output = match function.sig.output
	{
		ReturnType::Default => Tuple(TypeTuple
		{
			paren_token: Paren::default(),
			elems: Punctuated::new(),
		}),
		ReturnType::Type(_, t) => *t,
	};

	let error_doc = format!("The [error](std::error::Error) returned by [`{}`]", ident);
	let error_ident = format_ident!("{}Error", ident.to_string().to_camel_case());
	let error_variants = errors.iter().map(|(err, alias)| format_ident!("{}",
		if let Some(Lit::Str(alias)) = alias
		{
			alias.value()
		}
		else
		{
			err.segments.iter().map(|s| s.ident.to_string().replace("Error", "").to_camel_case()).collect()
		}
	));

	let errors_one = errors.keys();
	let errors_two = errors.keys();
	let error_variants_two = error_variants.clone();
	let error_variants_three = error_variants.clone();

	(quote!
	{
		#[doc = #error_doc]
		#[derive(Debug)]
		#vis enum #error_ident
		{
			#(#error_variants (#errors_one)),*
		}

		#[automatically_derived]
		impl core::fmt::Display for #error_ident
		{
			fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
			{
				match self
				{
					#(Self::#error_variants_two(e) => write!(f, "{}", e)),*
				}
			}
		}

		#(
			#[automatically_derived]
			impl std::convert::From<#errors_two> for #error_ident
			{
				fn from(e: #errors_two) -> Self
				{
					Self::#error_variants_three(e)
				}
			}
		)*

		#[automatically_derived]
		impl std::error::Error for #error_ident {}

		#(#attrs)* #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics (#(#inputs),* #variadic) #where_clause
			-> std::result::Result<#output, #error_ident>
		#block
	}).into()
}
