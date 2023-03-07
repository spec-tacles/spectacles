use std::io::stderr;

pub use rmp_serde::{
	encode::write_named as to_writer, from_read, from_slice, to_vec_named as to_vec,
};
pub use rmpv::{Value, ValueRef};
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

pub mod io;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event<T> {
	pub name: String,
	pub data: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventRef<'a, T> {
	pub name: &'a str,
	pub data: T,
}

pub type AnyEvent = Event<Value>;
pub type AnyEventRef<'a> = EventRef<'a, ValueRef<'a>>;

pub fn init_tracing() {
	tracing_subscriber::fmt::fmt()
		.with_writer(stderr)
		.with_env_filter(EnvFilter::from_default_env())
		.init();
}
