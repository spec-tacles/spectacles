use flexbuffers::FlexBufferType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event<T> {
	pub name: String,
	pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventRef<'a, T> {
	pub name: &'a str,
	pub data: T,
}

pub type AnyEvent = Event<FlexBufferType>;
pub type AnyEventRef<'a> = EventRef<'a, FlexBufferType>;
