use bson::{RawBson, RawBsonRef};
use serde::{Deserialize, Serialize};

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

pub type AnyEvent = Event<RawBson>;
pub type AnyEventRef<'a> = EventRef<'a, RawBsonRef<'a>>;
