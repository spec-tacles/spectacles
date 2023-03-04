use ::config::Config;
use anyhow::Result;
use futures::StreamExt;
use spectacles::{init_tracing, EventRef};
use tokio::io::{stdout, AsyncWriteExt};
use tracing::{debug, info, warn};
use twilight_gateway::{
	stream::{self, ShardEventStream},
	ConfigBuilder,
};
use twilight_http::Client;
use twilight_model::gateway::event::DispatchEvent;

use crate::config::Shards;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
	init_tracing();

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

	let gw_config = twilight_gateway::Config::new(config.token, config.gateway.intents);

	let per_shard_config = |_, mut builder: ConfigBuilder| {
		if let Some(event_types) = config.gateway.events.clone() {
			builder = builder.event_types(event_types.into_iter().map(Into::into).collect());
		}

		builder.build()
	};

	let mut shards: Vec<_> = match config.gateway.shards {
		Shards::Recommended => stream::create_recommended(&client, gw_config, per_shard_config)
			.await?
			.collect(),
		Shards::Bucket {
			bucket_id,
			concurrency,
			total,
		} => stream::create_bucket(bucket_id, concurrency, total, gw_config, per_shard_config)
			.collect(),
		Shards::Range { from, to, total } => {
			stream::create_range(from..to, total, gw_config, per_shard_config).collect()
		}
	};

	let mut stream = ShardEventStream::new(shards.iter_mut());

	let mut out = stdout();
	while let Some((shard, event)) = stream.next().await {
		match event {
			Ok(event) => {
				let kind = event.kind();

				debug!(kind = kind.name().unwrap_or("[unknown]"), shard = ?shard.id(), ?event);

				if let Ok(dispatch) = DispatchEvent::try_from(event) {
					let event = EventRef {
						name: kind.name().unwrap_or_default(),
						data: dispatch,
					};

					let bytes = bson::to_vec(&event)?;
					out.write_all(&bytes).await?;
					out.flush().await?;
				}
			}
			Err(error) => {
				warn!(?error);

				if error.is_fatal() {
					break;
				}
			}
		}
	}

	Ok(())
}
