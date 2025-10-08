use bunner_qs_rs::{
	OptionsValidationError, ParseOptions, Qs, QsParseError, QsStringifyError, StringifyOptions,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub type ParseResult<T> = Result<T, QsParseError>;
pub type StringifyResult = Result<String, QsStringifyError>;

pub fn build_parse_options<F>(configure: F) -> Result<ParseOptions, OptionsValidationError>
where
	F: FnOnce(ParseOptions) -> ParseOptions,
{
	let options = configure(ParseOptions::default());
	options.validate()?;
	Ok(options)
}

pub fn build_stringify_options<F>(configure: F) -> Result<StringifyOptions, OptionsValidationError>
where
	F: FnOnce(StringifyOptions) -> StringifyOptions,
{
	let options = configure(StringifyOptions::default());
	options.validate()?;
	Ok(options)
}

pub fn parse_default<T>(query: &str) -> ParseResult<T>
where
	T: DeserializeOwned + Default + 'static,
{
	parse_query(query, &ParseOptions::default())
}

pub fn parse_query<T>(query: &str, options: &ParseOptions) -> ParseResult<T>
where
	T: DeserializeOwned + Default + 'static,
{
	let qs = Qs::new()
		.with_parse(options.clone())
		.expect("parse options should validate");
	qs.parse(query)
}

pub fn stringify_default<T>(value: &T) -> StringifyResult
where
	T: Serialize,
{
	stringify_with_options(value, &StringifyOptions::default())
}

pub fn stringify_with_options<T>(value: &T, options: &StringifyOptions) -> StringifyResult
where
	T: Serialize,
{
	let qs = Qs::new()
		.with_stringify(options.clone())
		.expect("stringify options should validate");
	qs.stringify(value)
}
