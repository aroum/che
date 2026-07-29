yazi_macro::mod_pub!(archive sftp);

yazi_macro::mod_flat!(calculator copier descr dir_entry gate provider providers read_dir rw_file);

pub(super) fn init() { sftp::init(); }
