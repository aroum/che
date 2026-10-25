yazi_macro::mod_pub!(local);

yazi_macro::mod_flat!(attrs capabilities demand traits);

pub use demand::Demand as Gate;
pub use traits::Engine as Provider;
