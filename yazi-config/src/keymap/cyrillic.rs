pub(super) fn map_cyrillic(c: char) -> char {
	yazi_shared::translit::cyrillic_to_qwerty(c).unwrap_or(c)
}
