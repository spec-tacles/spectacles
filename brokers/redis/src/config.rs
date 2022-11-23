use anyhow::Result;
use clap::Parser;
use config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Parser)]
#[command(name = "spectacles-redis")]
pub struct Config {
	#[arg(long, short, env = "REDIS_CONFIG_FILE", exclusive = true)]
	pub config_file: Option<String>,

	#[arg(long, short, env = "REDIS_ADDRESS", default_value = "localhost:6379")]
	#[serde(default = "Config::default_address")]
	pub address: String,

	#[arg(
		long,
		short,
		env = "REDIS_GROUP",
		required_unless_present("config_file"),
		default_value = ""
	)]
	pub group: String,

	#[arg(long, short, env = "REDIS_EVENTS", value_delimiter = ',')]
	#[serde(default = "Config::default_events")]
	pub events: Vec<String>,
}

impl Config {
	pub fn default_address() -> String {
		"localhost:6379".to_string()
	}

	pub fn default_events() -> Vec<String> {
		vec![]
	}
}

impl Config {
	pub fn build() -> Result<Config> {
		let opt = Config::parse();

		if let Some(config_file) = opt.config_file {
			let file_source = config::File::with_name(&config_file).required(false);

			let env_source = config::Environment::with_prefix("REDIS")
				.try_parsing(true)
				.list_separator(",")
				.with_list_parse_key("events");

			let config: Config = config::Config::builder()
				.add_source(file_source)
				.add_source(env_source)
				.build()?
				.try_deserialize()?;

			return Ok(config);
		} else {
			Ok(opt)
		}
	}
}
