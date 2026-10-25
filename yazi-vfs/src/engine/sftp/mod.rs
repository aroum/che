yazi_macro::mod_flat!(absolute conn demand metadata read_dir sftp);

pub use demand::Demand as Gate;

static CONN: yazi_shared::RoCell<
	parking_lot::Mutex<
		hashbrown::HashMap<
			&'static yazi_config::vfs::ServiceSftp,
			&'static deadpool::managed::Pool<Conn>,
		>,
	>,
> = yazi_shared::RoCell::new();

pub(super) fn init() { CONN.init(Default::default()); }
