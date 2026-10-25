yazi_macro::mod_pub!(archive sftp);

yazi_macro::mod_flat!(calculator copier demand descr dir_entry engine engines read_dir rw_file);

pub use demand::Demand as Gate;
pub use engines::Engines as Providers;

pub(super) fn init() { sftp::init(); }
