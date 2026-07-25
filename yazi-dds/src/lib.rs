mod macros;

yazi_macro::mod_pub!(ember spark);

yazi_macro::mod_flat!(client payload pubsub pump sendable server state stream);

pub fn init() {
	let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

	// Client
	ID.init(yazi_boot::ARGS.client_id.unwrap_or(yazi_shared::Id::unique()));
	PEERS.with(<_>::default);
	QUEUE_TX.init(tx);
	QUEUE_RX.init(rx);

	// Server
	CLIENTS.with(<_>::default);
	STATE.with(<_>::default);

	// Pubsub
	LOCAL.with(<_>::default);
	REMOTE.with(<_>::default);

	unsafe {
		if let Some(s) =
			std::env::var("CHE_ID").or_else(|_| std::env::var("GEZI_ID")).ok().filter(|s| !s.is_empty())
		{
			std::env::set_var("CHE_PID", &s);
			std::env::set_var("GEZI_PID", &s);
		}
		let id_str = ID.to_string();
		std::env::set_var("CHE_ID", &id_str);
		std::env::set_var("GEZI_ID", &id_str);
		let level = (std::env::var("CHE_LEVEL")
			.or_else(|_| std::env::var("GEZI_LEVEL"))
			.unwrap_or_default()
			.parse()
			.unwrap_or(0u16)
			+ 1)
			.to_string();
		std::env::set_var("CHE_LEVEL", &level);
		std::env::set_var("GEZI_LEVEL", &level);
	}
}

pub fn serve() {
	Pump::serve();
	Client::serve();
}

pub async fn shutdown() {
	Pump::shutdown().await;
}
