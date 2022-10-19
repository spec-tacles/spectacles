use std::{
	io::{stderr, stdout, Write},
	sync::Arc,
};

use ::config::Config;
use anyhow::Result;
use capnp::{message::TypedBuilder, serialize_packed};
use futures::StreamExt;
use tokio::spawn;
use tracing::info;
use twilight_gateway::Cluster;
use twilight_http::Client;
use twilight_model::gateway::event::DispatchEvent;

use crate::schema::gateway_capnp::packet;

mod config;
mod schema;

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::fmt().with_writer(stderr).init();

	let config: config::Config = Config::builder()
		.add_source(::config::File::with_name("gateway"))
		.add_source(::config::Environment::default())
		.build()?
		.try_deserialize()?;

	info!("{:?}", config);

	let mut builder = Client::builder();

	if let Some(base) = config.api.base {
		builder = builder.proxy(base.url, base.use_http);
	}

	let client = builder
		.timeout(config.api.timeout)
		.token(config.token.clone())
		.build();

	let mut builder = Cluster::builder(config.token, config.gateway.intents);

	if let Some(event_types) = config.gateway.events {
		builder = builder.event_types(event_types.into_iter().map(Into::into).collect());
	}

	if let Some(shards) = config.gateway.shards {
		builder = builder.shard_scheme(shards.into());
	}

	let (cluster, mut events) = builder.http_client(Arc::new(client)).build().await?;

	spawn(async move {
		cluster.up().await;
	});

	let mut out = stdout();
	while let Some((shard, event)) = events.next().await {
		let kind = event.kind();
		if let Ok(dispatch) = DispatchEvent::try_from(event) {
			let d = serde_json::to_vec(&dispatch)?;

			let mut message = TypedBuilder::<packet::Owned, _>::new_default();
			let mut packet = message.init_root();
			packet.set_d(&d);
			packet.set_t(kind.name().unwrap_or_default());
			packet.set_shard(shard);

			serialize_packed::write_message(&mut out, message.borrow_inner())?;
			out.flush()?;
		}
	}

	Ok(())
}
