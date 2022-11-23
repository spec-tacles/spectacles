use anyhow::Result;
use serde::{Deserialize, Serialize};
use clap::{Parser};
use config;

#[derive(Debug, Serialize, Deserialize, Parser)]
#[command(name = "spectacles-redis")]
pub struct Config {
	#[arg(long, short, env = "REDIS_CONFIG_FILE", exclusive = true)]
	pub config_file: Option<String>,

	#[arg(long, short, env = "REDIS_ADDRESS", default_value = "localhost:6379")]
	pub address: String,

	#[arg(long, short, env = "REDIS_GROUP", required_unless_present("config_file"), default_value = "")]
	pub group: String,

	#[arg(long, short, env = "REDIS_EVENTS", value_delimiter = ',')]
	pub events: Vec<String>,
}

impl Config {
	pub fn build() -> Result<Config> {
		let opt = Config::parse();

		if let Some(config_file) = opt.config_file {
			let file_source = config::File::with_name(&config_file)
				.required(false);

			let env_source = config::Environment::with_prefix("REDIS")
				.try_parsing(true)
				.list_separator(",")
				.with_list_parse_key("events");

			let config: Config = config::Config::builder()
				.add_source(file_source)
				.add_source(env_source)
				.set_default("address", "localhost:6379")?
				.set_default("events", vec![] as Vec<String>)?
				.build()?
				.try_deserialize()?;

			return Ok(config);
		} else {
			Ok(opt)
		}
	}
}
