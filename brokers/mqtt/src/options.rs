use std::{path::PathBuf, time::Duration};

use humantime::parse_duration;
use paho_mqtt::{
	ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder, Error, SslOptions,
	SslOptionsBuilder,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "spectacles-mqtt", about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Opt {
	#[structopt(flatten)]
	pub create: CreateOpt,
	#[structopt(flatten)]
	pub connect: ConnectOpt,
}

#[derive(Debug, StructOpt)]
pub struct CreateOpt {
	/// The URL of the MQTT server.
	#[structopt(env, short, long)]
	pub url: String,

	/// The client ID useful for session resuming
	#[structopt(long, default_value)]
	pub client_id: String,

	/// The MQTT version
	#[structopt(long, short = "v", default_value = "5")]
	pub mqtt_version: u32,
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

#[derive(Debug, StructOpt)]
pub struct ConnectOpt {
	/// The keep-alive interval for the client session.
	#[structopt(long, parse(try_from_str = parse_duration))]
	pub keep_alive_interval: Option<Duration>,

	/// Sets the 'clean session' flag to send to the broker.
	///
	/// This is for MQTT v3.x connections only, and if set, will set the other options to be
	/// compatible with v3.
	#[structopt(long)]
	pub clean_session: bool,

	/// Sets the 'clean start' flag to send to the broker.
	///
	/// This is for MQTT v5 connections only, and if set, will set the other options to be compatible
	/// with v5.
	#[structopt(long)]
	pub clean_start: bool,

	/// The maximum number of in-flight messages that can be simultaneously handled by this client.
	#[structopt(long)]
	pub max_inflight: Option<i32>,

	/// The username for authentication with the broker.
	#[structopt(long, short)]
	pub username: Option<String>,

	/// The password for authenticaton with the broker.
	#[structopt(long, short)]
	pub password: Option<String>,

	/// The time interval in which to allow the connection to complete.
	#[structopt(long, parse(try_from_str = parse_duration))]
	pub connect_timeout: Option<Duration>,

	/// The time interval in which to retry connections.
	#[structopt(long, parse(try_from_str = parse_duration))]
	pub retry_interval: Option<Duration>,

	/// The minimum interval in which to retry connecting.
	#[structopt(long, parse(try_from_str = parse_duration), required_if("automatic_reconnect_max", "Option::is_some"))]
	pub automatic_reconnect_min: Option<Duration>,

	/// The maximum interval in which to retry connecting.
	#[structopt(long, parse(try_from_str = parse_duration), required_if("automatic_reconnect_min", "Option::is_some"))]
	pub automatic_reconnect_max: Option<Duration>,

	/// The HTTP proxy for websockets.
	#[structopt(long)]
	pub http_proxy: Option<String>,

	/// The HTTPS proxy for websockets.
	#[structopt(long)]
	pub https_proxy: Option<String>,

	#[structopt(flatten)]
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

#[derive(Debug, StructOpt)]
pub struct SslOpts {
	/// Path to the PEM file containing public certificates to trust.
	#[structopt(long)]
	pub trust_store: Option<PathBuf>,

	/// Path to the PEM file containing the public certificate chain.
	#[structopt(long)]
	pub key_store: Option<PathBuf>,

	/// Path to the PEM file containing the client's private key if not in the key store.
	#[structopt(long)]
	pub private_key: Option<PathBuf>,

	/// The password to load the private key, if encrypted.
	#[structopt(long)]
	pub private_key_password: Option<String>,

	/// The list of cypher suites the client will present to the server during the SSL handshake.
	///
	/// For a full explanation of the cipher list format, please see the OpenSSL on-line
	/// documentation: http://www.openssl.org/docs/apps/ciphers.html#CIPHER_LIST_FORMAT
	///
	/// If this setting is ommitted, its default value will be “ALL”, that is, all the cipher suites
	/// -excluding those offering no encryption- will be considered.
	#[structopt(long)]
	pub enabled_cypher_suites: Option<String>,

	/// Whether verification of the server certificate is enabled.
	#[structopt(long)]
	pub enable_server_cert_auth: bool,

	/// Whether to perform post connection certificate checks.
	#[structopt(long)]
	pub verify: bool,

	/// Path to the directory containing CA certificates in PEM format.
	#[structopt(long)]
	pub ca_path: Option<PathBuf>,

	/// Whether to load the default SSL CA.
	#[structopt(long)]
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

		if let Some(enabled_cipher_suites) = opts.enabled_cypher_suites {
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
