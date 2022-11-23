use anyhow::Result;
use clap::Parser;
use config;
use humantime::parse_duration;
use paho_mqtt::{
	ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder, Error, SslOptions,
	SslOptionsBuilder,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Serialize, Deserialize, Parser)]
#[command(name = "spectacles-redis", about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Config {
	#[arg(long, short, env = "MQTT_CONFIG_FILE", exclusive = true)]
	pub config_file: Option<String>,

	#[command(flatten)]
	#[serde(flatten)]
	pub create: CreateOpt,

	#[command(flatten)]
	pub connect: ConnectOpt,

	/// Events to subscribe to.
	#[arg(short, long, env = "MQTT_EVENTS", value_delimiter = ',')]
	#[serde(default)]
	pub events: Vec<String>,

	/// Quality of Service for sending & receiving messages
	/// - 0: At most once
	/// - 1: At least once
	/// - 2: Exactly once
	#[arg(long, env = "MQTT_QOS", default_value = "2")]
	#[serde(default = "Config::default_qos")]
	pub qos: i32,
}

impl Config {
	pub fn default_qos() -> i32 {
		2
	}

	pub fn build() -> Result<Config> {
		let opt = Config::parse();

		if let Some(config_file) = opt.config_file {
			let file_source = config::File::with_name(&config_file).required(false);

			let env_source = config::Environment::with_prefix("MQTT")
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

#[derive(Debug, Serialize, Deserialize, Parser)]
pub struct CreateOpt {
	/// The URL of the MQTT server.
	#[arg(long, env = "MQTT_URL", default_value = "localhost:1883")]
	#[serde(default = "CreateOpt::default_url")]
	pub url: String,

	/// The client ID useful for session resuming
	#[arg(long, env = "MQTT_CLIENT_ID", default_value = "")]
	#[serde(default)]
	pub client_id: String,

	/// The MQTT version
	#[arg(long, short = 'v', env = "MQTT_VERSION", default_value = "5")]
	#[serde(default = "CreateOpt::default_version")]
	pub mqtt_version: u32,
}

impl CreateOpt {
	pub fn default_url() -> String {
		"localhost:1883".to_string()
	}

	pub fn default_version() -> u32 {
		5
	}
}

impl From<CreateOpt> for CreateOptions {
	fn from(opt: CreateOpt) -> Self {
		CreateOptionsBuilder::new()
			.server_uri(opt.url)
			.client_id(opt.client_id)
			.mqtt_version(opt.mqtt_version)
			.finalize()
	}
}

#[derive(Debug, Serialize, Deserialize, Parser)]
pub struct ConnectOpt {
	/// The keep-alive interval for the client session.
	#[arg(long, env = "MQTT_KEEP_ALIVE_INTERVAL", value_parser(parse_duration))]
	pub keep_alive_interval: Option<Duration>,

	/// Sets the 'clean session' flag to send to the broker.
	///
	/// This is for MQTT v3.x connections only, and if set, will set the other options to be
	/// compatible with v3.
	#[arg(long, env = "MQTT_CLEAN_SESSION")]
	#[serde(default)]
	pub clean_session: bool,

	/// Sets the 'clean start' flag to send to the broker.
	///
	/// This is for MQTT v5 connections only, and if set, will set the other options to be compatible
	/// with v5.
	#[arg(long, env = "MQTT_CLEAN_START")]
	#[serde(default)]
	pub clean_start: bool,

	/// The maximum number of in-flight messages that can be simultaneously handled by this client.
	#[arg(long, env = "MQTT_MAX_INFLIGHT")]
	pub max_inflight: Option<i32>,

	/// The username for authentication with the broker.
	#[arg(long, short, env = "MQTT_USERNAME")]
	pub username: Option<String>,

	/// The password for authenticaton with the broker.
	#[arg(long, short, env = "MQTT_PASSWORD")]
	pub password: Option<String>,

	/// The time interval in which to allow the connection to complete.
	#[arg(long, env = "MQTT_CONNECT_TIMEOUT", value_parser(parse_duration))]
	pub connect_timeout: Option<Duration>,

	/// The time interval in which to retry connections.
	#[arg(long, env = "MQTT_RETRY_INTERVAL", value_parser(parse_duration))]
	pub retry_interval: Option<Duration>,

	/// The minimum interval in which to retry connecting.
	#[arg(
		long,
		env = "MQTT_AUTOMATIC_RECONNECT_MIN",
		value_parser(parse_duration),
		requires("automatic_reconnect_max")
	)]
	pub automatic_reconnect_min: Option<Duration>,

	/// The maximum interval in which to retry connecting.
	#[arg(
		long,
		env = "MQTT_AUTOMATIC_RECONNECT_MAX",
		value_parser(parse_duration),
		requires("automatic_reconnect_min")
	)]
	pub automatic_reconnect_max: Option<Duration>,

	/// The HTTP proxy for websockets.
	#[arg(long, env = "MQTT_HTTP_PROXY")]
	pub http_proxy: Option<String>,

	/// The HTTPS proxy for websockets.
	#[arg(long, env = "MQTT_HTTPS_PROXY")]
	pub https_proxy: Option<String>,

	#[command(flatten)]
	pub ssl: SslOpts,
}

impl TryFrom<ConnectOpt> for ConnectOptions {
	type Error = Error;

	fn try_from(opt: ConnectOpt) -> Result<Self, Self::Error> {
		let mut builder = ConnectOptionsBuilder::new();
		builder
			.clean_session(opt.clean_session)
			.clean_start(opt.clean_start)
			.ssl_options(opt.ssl.try_into()?);

		if let Some(keep_alive_interval) = opt.keep_alive_interval {
			builder.keep_alive_interval(keep_alive_interval);
		}

		if let Some(max_inflight) = opt.max_inflight {
			builder.max_inflight(max_inflight);
		}

		if let Some(username) = opt.username {
			builder.user_name(username);
		}

		if let Some(password) = opt.password {
			builder.password(password);
		}

		if let Some(connect_timeout) = opt.connect_timeout {
			builder.connect_timeout(connect_timeout);
		}

		if let Some(retry_interval) = opt.retry_interval {
			builder.retry_interval(retry_interval);
		}

		if let Some((min, max)) = opt.automatic_reconnect_min.zip(opt.automatic_reconnect_max) {
			builder.automatic_reconnect(min, max);
		}

		if let Some(http_proxy) = opt.http_proxy {
			builder.http_proxy(http_proxy);
		}

		if let Some(https_proxy) = opt.https_proxy {
			builder.https_proxy(https_proxy);
		}

		Ok(builder.finalize())
	}
}

#[derive(Debug, Serialize, Deserialize, Parser)]
pub struct SslOpts {
	/// Path to the PEM file containing public certificates to trust.
	#[arg(long, env = "MQTT_TRUST_STORE")]
	pub trust_store: Option<PathBuf>,

	/// Path to the PEM file containing the public certificate chain.
	#[arg(long, env = "MQTT_KEY_STORE")]
	pub key_store: Option<PathBuf>,

	/// Path to the PEM file containing the client's private key if not in the key store.
	#[arg(long, env = "MQTT_PRIVATE_KEY")]
	pub private_key: Option<PathBuf>,

	/// The password to load the private key, if encrypted.
	#[arg(long, env = "MQTT_PRIVATE_KEY_PASSWORD")]
	pub private_key_password: Option<String>,

	/// The list of cypher suites the client will present to the server during the SSL handshake.
	///
	/// For a full explanation of the cipher list format, please see the OpenSSL on-line
	/// documentation: http://www.openssl.org/docs/apps/ciphers.html#CIPHER_LIST_FORMAT
	///
	/// If this setting is ommitted, its default value will be “ALL”, that is, all the cipher suites
	/// -excluding those offering no encryption- will be considered.
	#[arg(long, env = "MQTT_ENABLED_CIPHER_SUITES")]
	pub enabled_cipher_suites: Option<String>,

	/// Whether verification of the server certificate is enabled.
	#[arg(long, env = "MQTT_ENABLE_SERVER_CERT_AUTH")]
	#[serde(default)]
	pub enable_server_cert_auth: bool,

	/// Whether to perform post connection certificate checks.
	#[arg(long, env = "MQTT_VERIFY")]
	#[serde(default)]
	pub verify: bool,

	/// Path to the directory containing CA certificates in PEM format.
	#[arg(long, env = "MQTT_CA_PATH")]
	pub ca_path: Option<PathBuf>,

	/// Whether to load the default SSL CA.
	#[arg(long, env = "MQTT_DISABLE_DEFAULT_TRUST_STORE")]
	#[serde(default)]
	pub disable_default_trust_store: bool,
}

impl TryFrom<SslOpts> for SslOptions {
	type Error = Error;

	fn try_from(opts: SslOpts) -> Result<Self, Self::Error> {
		let mut builder = &mut SslOptionsBuilder::new();

		if let Some(trust_store) = opts.trust_store {
			builder = builder.trust_store(trust_store)?;
		}

		if let Some(key_store) = opts.key_store {
			builder = builder.key_store(key_store)?;
		}

		if let Some(private_key) = opts.private_key {
			builder = builder.private_key(private_key)?;
		}

		if let Some(private_key_password) = opts.private_key_password {
			builder = builder.private_key_password(private_key_password);
		}

		if let Some(enabled_cipher_suites) = opts.enabled_cipher_suites {
			builder = builder.enabled_cipher_suites(enabled_cipher_suites);
		}

		if let Some(ca_path) = opts.ca_path {
			builder = builder.ca_path(ca_path)?;
		}

		builder = builder
			.enable_server_cert_auth(opts.enable_server_cert_auth)
			.verify(opts.verify)
			.disable_default_trust_store(opts.disable_default_trust_store);

		Ok(builder.finalize())
	}
}
