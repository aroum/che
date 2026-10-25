#[macro_export]
macro_rules! time {
	($expr:expr) => {
		$crate::time!(stringify!($expr), $expr)
	};
	($label:expr, $expr:expr) => {
		$crate::time!($expr, "{}", $label)
	};
	($expr:expr, $fmt:expr, $($args:tt)*) => {{
		if tracing::enabled!(tracing::Level::DEBUG) {
			let start = std::time::Instant::now();
			let result = $expr;
			tracing::debug!("{} took {:?}", format_args!($fmt, $($args)*), start.elapsed());
			result
		} else {
			$expr
		}
	}};
}

#[macro_export]
macro_rules! debug {
	($($args:tt)+) => { tracing::debug!($($args)+) };
}

#[macro_export]
macro_rules! warn {
	($($args:tt)+) => { tracing::warn!($($args)+) };
}

#[macro_export]
macro_rules! error {
	($($args:tt)+) => { tracing::error!($($args)+) };
}

#[macro_export]
macro_rules! err {
	($expr:expr) => {
		$crate::err!(stringify!($expr), $expr)
	};
	($label:expr, $expr:expr) => {
		$crate::err!($expr, "{}", $label)
	};
	($expr:expr, $fmt:expr, $($args:tt)*) => {{
		if let Err(e) = $expr {
			tracing::error!("{} failed: {e}", format_args!($fmt, $($args)*));
		}
	}};
}
