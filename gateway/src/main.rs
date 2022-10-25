use std::{
	io::{stderr, stdout, Write},
	sync::Arc,
};

use ::config::Config;
use anyhow::Result;
use futures::StreamExt;
use spectacles::EventRef;
use tokio::spawn;
use tracing::{debug, info};
use twilight_gateway::Cluster;
use twilight_http::Client;
use twilight_model::gateway::event::DispatchEvent;

mod config;

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

		debug!(kind = kind.name().unwrap_or("[unknown]"), shard, ?event);

		if let Ok(dispatch) = DispatchEvent::try_from(event) {
			let event = EventRef {
				name: kind.name().unwrap_or_default(),
				data: dispatch,
			};

			let bytes = bson::to_vec(&event)?;
			out.write_all(&bytes)?;
			out.flush()?;
		}
	}

	Ok(())
}
