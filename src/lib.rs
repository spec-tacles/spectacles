use bincode::{BorrowDecode, Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
pub struct Event<T> {
	pub name: String,
	pub data: T,
}

#[derive(Debug, Deserialize, Serialize, Encode, BorrowDecode)]
pub struct EventRef<'a, T> {
	pub name: &'a str,
	pub data: T,
}
