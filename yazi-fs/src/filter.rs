use std::{fmt::Display, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::{event::Action, strand::AsStrand};

use super::Normalizer;

pub struct Filter {
	raw:   String,
	regex: Regex,
}

fn glob_to_regex(s: &str) -> String {
	if s.contains('*') || s.contains('?') {
		let mut res = String::with_capacity(s.len() * 2);
		for c in s.chars() {
			match c {
				'*' => res.push_str(".*"),
				'?' => res.push('.'),
				'.' | '(' | ')' | '[' | ']' | '{' | '}' | '+' | '^' | '$' | '|' | '\\' => {
					res.push('\\');
					res.push(c);
				}
				_ => res.push(c),
			}
		}
		res
	} else {
		s.to_owned()
	}
}

impl Filter {
	pub fn new(s: &str, case: FilterCase) -> Result<Self> {
		let glob_pat = glob_to_regex(s);
		let pat = Normalizer::normalize(&glob_pat)?;
		let regex = match case {
			FilterCase::Smart => {
				let uppercase = s.chars().any(|c| c.is_uppercase());
				RegexBuilder::new(&pat).case_insensitive(!uppercase).build()?
			}
			FilterCase::Sensitive => Regex::new(&pat)?,
			FilterCase::Insensitive => RegexBuilder::new(&pat).case_insensitive(true).build()?,
		};
		Ok(Self { raw: s.to_owned(), regex })
	}

	#[inline]
	#[allow(private_bounds)]
	pub fn matches<T>(&self, name: T) -> bool
	where
		T: AsStrand,
	{
		self.regex.is_match(name.as_strand().encoded_bytes())
	}

	#[inline]
	pub fn highlighted(&self, name: impl AsStrand) -> Option<Vec<Range<usize>>> {
		self.regex.find(name.as_strand().encoded_bytes()).map(|m| vec![m.range()])
	}
}

impl PartialEq for Filter {
	fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}

impl Display for Filter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.raw) }
}

// --- FilterCase
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FilterCase {
	Smart,
	#[default]
	Sensitive,
	Insensitive,
}

impl From<&Action> for FilterCase {
	fn from(a: &Action) -> Self {
		match (a.bool("smart"), a.bool("insensitive")) {
			(true, _) => Self::Smart,
			(_, false) => Self::Sensitive,
			(_, true) => Self::Insensitive,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_glob_filter_wildcards() {
		let filter = Filter::new("*.sh", FilterCase::Smart).unwrap();
		assert!(filter.matches("test.sh"));
		assert!(filter.matches("build_app.sh"));
		assert!(!filter.matches("test.py"));
		assert!(!filter.matches("README.md"));

		let filter_question = Filter::new("file?.txt", FilterCase::Smart).unwrap();
		assert!(filter_question.matches("file1.txt"));
		assert!(filter_question.matches("fileA.txt"));
		assert!(!filter_question.matches("file12.txt"));
	}

	#[test]
	fn test_glob_filter_case_flags() {
		// Smart case (lowercase pattern -> case insensitive)
		let smart_lower = Filter::new("*.sh", FilterCase::Smart).unwrap();
		assert!(smart_lower.matches("script.sh"));
		assert!(smart_lower.matches("SCRIPT.SH"));

		// Smart case (uppercase pattern -> case sensitive)
		let smart_upper = Filter::new("*.SH", FilterCase::Smart).unwrap();
		assert!(smart_upper.matches("SCRIPT.SH"));
		assert!(!smart_upper.matches("script.sh"));

		// Sensitive case
		let sensitive = Filter::new("*.sh", FilterCase::Sensitive).unwrap();
		assert!(sensitive.matches("script.sh"));
		assert!(!sensitive.matches("SCRIPT.SH"));

		// Insensitive case
		let insensitive = Filter::new("*.SH", FilterCase::Insensitive).unwrap();
		assert!(insensitive.matches("script.sh"));
		assert!(insensitive.matches("SCRIPT.SH"));
	}

	#[test]
	fn test_cyrillic_filter() {
		// Smart case with Russian lowercase matches uppercase and lowercase
		let filter = Filter::new("отчет", FilterCase::Smart).unwrap();
		assert!(filter.matches("отчет_2026.docx"));
		assert!(filter.matches("ОТЧЕТ_ГОДОВОЙ.PDF"));
		assert!(filter.matches("Отчет.txt"));
		assert!(!filter.matches("документ.txt"));

		// Smart case with Russian uppercase is case-sensitive
		let filter_upper = Filter::new("Отчет", FilterCase::Smart).unwrap();
		assert!(filter_upper.matches("Отчет.txt"));
		assert!(!filter_upper.matches("отчет_2026.docx"));

		// Russian glob filter
		let filter_glob = Filter::new("*док*.txt", FilterCase::Smart).unwrap();
		assert!(filter_glob.matches("новый_документ.txt"));
		assert!(filter_glob.matches("ДОКУМЕНТ_1.TXT"));
		assert!(!filter_glob.matches("новый_документ.docx"));
	}
}
