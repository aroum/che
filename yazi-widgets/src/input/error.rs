use std::{error::Error, fmt::{self, Display}};

use yazi_shared::Id;

#[derive(Debug)]
pub enum InputError {
	Typed(String),
	Completed(String, Id),
	Canceled(String),
	Arrow(String, isize),
}

impl Display for InputError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Typed(text) => write!(f, "Typed error: {text}"),
			Self::Completed(text, _) => write!(f, "Completed error: {text}"),
			Self::Canceled(text) => write!(f, "Canceled error: {text}"),
			Self::Arrow(text, step) => write!(f, "Arrow error: {text} ({step})"),
		}
	}
}

impl Error for InputError {}
