use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};
use twilight_gateway::{EventType, Intents};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub token: String,
	pub gateway: Gateway,
	#[serde(default)]
	pub api: Api,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gateway {
	pub intents: Intents,
	#[serde(default)]
	pub events: Option<Vec<EventType>>,
	#[serde(default)]
	pub shards: Shards,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Shards {
	Bucket {
		bucket_id: u64,
		concurrency: u64,
		total: u64,
	},
	Range {
		from: u64,
		to: u64,
		total: u64,
	},
	Recommended,
}

impl Default for Shards {
	fn default() -> Self {
		Self::Recommended
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Api {
	#[serde(default)]
	pub version: Option<NonZeroUsize>,
	#[serde(default)]
	pub base: Option<ApiBase>,
	#[serde(default = "Api::default_timeout")]
	pub timeout: Duration,
}

impl Api {
	const fn default_timeout() -> Duration {
		Duration::from_secs(10)
	}
}

impl Default for Api {
	fn default() -> Self {
		Self {
			version: None,
			base: None,
			timeout: Api::default_timeout(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiBase {
	pub url: String,
	pub use_http: bool,
}
