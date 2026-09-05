use super::Transliterator;

pub fn cyrillic_to_qwerty(c: char) -> Option<char> {
	match c {
		// Lowercase
		'й' => Some('q'),
		'ц' => Some('w'),
		'у' => Some('e'),
		'к' => Some('r'),
		'е' => Some('t'),
		'н' => Some('y'),
		'г' => Some('u'),
		'ш' => Some('i'),
		'щ' => Some('o'),
		'з' => Some('p'),
		'х' => Some('['),
		'ъ' => Some(']'),
		'ф' => Some('a'),
		'ы' => Some('s'),
		'в' => Some('d'),
		'а' => Some('f'),
		'п' => Some('g'),
		'р' => Some('h'),
		'о' => Some('j'),
		'л' => Some('k'),
		'д' => Some('l'),
		'ж' => Some(';'),
		'э' => Some('\''),
		'я' => Some('z'),
		'ч' => Some('x'),
		'с' => Some('c'),
		'м' => Some('v'),
		'и' => Some('b'),
		'т' => Some('n'),
		'ь' => Some('m'),
		'б' => Some(','),
		'ю' => Some('.'),
		'ё' => Some('`'),

		// Uppercase
		'Й' => Some('Q'),
		'Ц' => Some('W'),
		'У' => Some('E'),
		'К' => Some('R'),
		'Е' => Some('T'),
		'Н' => Some('Y'),
		'Г' => Some('U'),
		'Ш' => Some('I'),
		'Щ' => Some('O'),
		'З' => Some('P'),
		'Х' => Some('{'),
		'Ъ' => Some('}'),
		'Ф' => Some('A'),
		'Ы' => Some('S'),
		'В' => Some('D'),
		'А' => Some('F'),
		'П' => Some('G'),
		'Р' => Some('H'),
		'О' => Some('J'),
		'Л' => Some('K'),
		'Д' => Some('L'),
		'Ж' => Some(':'),
		'Э' => Some('"'),
		'Я' => Some('Z'),
		'Ч' => Some('X'),
		'С' => Some('C'),
		'М' => Some('V'),
		'И' => Some('B'),
		'Т' => Some('N'),
		'Ь' => Some('M'),
		'Б' => Some('<'),
		'Ю' => Some('>'),
		'Ё' => Some('~'),

		_ => None,
	}
}

pub fn qwerty_to_cyrillic(c: char) -> Option<char> {
	match c.to_ascii_lowercase() {
		'q' => Some('й'),
		'w' => Some('ц'),
		'e' => Some('у'),
		'r' => Some('к'),
		't' => Some('е'),
		'y' => Some('н'),
		'u' => Some('г'),
		'i' => Some('ш'),
		'o' => Some('щ'),
		'p' => Some('з'),
		'[' => Some('х'),
		']' => Some('ъ'),
		'a' => Some('ф'),
		's' => Some('ы'),
		'd' => Some('в'),
		'f' => Some('а'),
		'g' => Some('п'),
		'h' => Some('р'),
		'j' => Some('о'),
		'k' => Some('л'),
		'l' => Some('д'),
		';' => Some('ж'),
		'\'' => Some('э'),
		'z' => Some('я'),
		'x' => Some('ч'),
		'c' => Some('с'),
		'v' => Some('м'),
		'b' => Some('и'),
		'n' => Some('т'),
		'm' => Some('ь'),
		',' => Some('б'),
		'.' => Some('ю'),
		'`' => Some('ё'),
		_ => None,
	}
}

pub fn latin_to_cyrillic_phonetic(c: char) -> Option<char> {
	match c.to_ascii_lowercase() {
		'a' => Some('а'),
		'b' => Some('б'),
		'v' | 'w' => Some('в'),
		'g' => Some('г'),
		'd' => Some('д'),
		'e' => Some('е'),
		'z' => Some('з'),
		'i' => Some('и'),
		'j' => Some('й'),
		'k' => Some('к'),
		'l' => Some('л'),
		'm' => Some('м'),
		'n' => Some('н'),
		'o' => Some('о'),
		'p' => Some('п'),
		'r' => Some('р'),
		's' => Some('с'),
		't' => Some('т'),
		'u' => Some('у'),
		'f' => Some('ф'),
		'h' => Some('х'),
		'c' => Some('ц'),
		'y' => Some('ы'),
		_ => None,
	}
}

pub fn matches_jump_char(name: &str, ch: char) -> bool {
	let Some(first_char) = name.chars().next() else { return false };
	let first_lower = first_char.to_lowercase().next().unwrap_or(first_char);
	let ch_lower = ch.to_lowercase().next().unwrap_or(ch);

	// 1. Exact / case-insensitive character match
	if first_lower == ch_lower {
		return true;
	}

	// 2. QWERTY keyboard match (e.g. key 'g' pressed on Russian layout matches 'п')
	if let Some(cyr) = qwerty_to_cyrillic(ch_lower)
		&& cyr == first_lower
	{
		return true;
	}

	// 3. Cyrillic to QWERTY keyboard match (e.g. key 'п' typed matches Latin 'g')
	if let Some(qwerty) = cyrillic_to_qwerty(ch_lower)
		&& qwerty.to_ascii_lowercase() == first_lower
	{
		return true;
	}

	// 4. Phonetic transliteration match (e.g. 'p' matches 'п', 'd' matches 'д')
	if let Some(cyr_phonetic) = latin_to_cyrillic_phonetic(ch_lower)
		&& cyr_phonetic == first_lower
	{
		return true;
	}

	// 5. Transliteration match (e.g. transliterated filename starts with the character)
	let name_bytes = name.as_bytes();
	let transliterated = name_bytes.transliterate().to_lowercase();
	if transliterated.starts_with(&ch_lower.to_string()) {
		return true;
	}

	false
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_matches_jump_char_cyrillic() {
		// Exact Russian char
		assert!(matches_jump_char("Привет.txt", 'п'));
		assert!(matches_jump_char("привет.txt", 'П'));

		// QWERTY keyboard match (key 'g' is 'п' in Russian layout)
		assert!(matches_jump_char("Привет.txt", 'g'));
		assert!(matches_jump_char("привет.txt", 'G'));

		// Phonetic match (Latin 'p' is 'п')
		assert!(matches_jump_char("Привет.txt", 'p'));
		assert!(matches_jump_char("привет.txt", 'P'));

		// Russian 'д' matches 'l' (QWERTY), 'd' (phonetic), and 'д' (exact)
		assert!(matches_jump_char("Документы", 'д'));
		assert!(matches_jump_char("документы", 'l'));
		assert!(matches_jump_char("документы", 'd'));

		// English filename with Russian keyboard
		assert!(matches_jump_char("documents", 'd'));
		assert!(matches_jump_char("documents", 'в'));
	}
}
