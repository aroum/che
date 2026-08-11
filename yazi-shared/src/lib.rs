#![allow(clippy::wrong_self_convention)]

yazi_macro::mod_pub!(data errors event loc path pool scheme shell strand translit url wtf8);

yazi_macro::mod_flat!(alias bytes chars completion_token condition debounce env id last_value layer localset natsort os predictor ro_cell source sync_cell terminal tests throttle time undo_op utf8);

pub fn init() {
	LOCAL_SET.with(tokio::task::LocalSet::new);

	LOG_LEVEL.replace(<_>::from(
		std::env::var("CHE_LOG").or_else(|_| std::env::var("GEZI_LOG")).unwrap_or_default(),
	));

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	pool::init();
	event::Event::init();
}
