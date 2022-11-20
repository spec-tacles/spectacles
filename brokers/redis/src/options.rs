use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Opt {
	pub address: String,
	pub group: String,
	pub events: Vec<String>,
}
