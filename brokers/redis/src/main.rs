use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use redust::pool::{Manager, Pool};
use spectacles::{init_tracing, io::read, to_vec, AnyEvent, EventRef};
use tokio::{
	io::{stdout, AsyncWriteExt},
	task::JoinSet,
};

use crate::client::Client;
use crate::config::Config;

mod client;
mod config;

async fn publish_from_stdin(client: Client) -> Result<()> {
	let mut stream = read::<AnyEvent>();
	while let Some(event) = stream.next().await {
		client.publish(event.name, &event.data).await?;
	}

	Ok(())
}

async fn consume_to_stdout(client: Client, events: Vec<String>) -> Result<()> {
	let mut out = stdout();
	let mut stream = client.consume(events);
	while let Some(message) = stream.try_next().await? {
		out.write_all(&to_vec(&EventRef {
			data: message.data.clone(),
			name: &String::from_utf8_lossy(&message.event),
		})?)
		.await?;

		message.ack().await?;
	}

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	init_tracing();

	let config = Config::build()?;

	let manager = Manager::new(config.address);
	let pool = Pool::builder(manager).build()?;
	let client = Client::new(config.group, pool);

	client.ensure_events(config.events.iter()).await?;

	let mut set = JoinSet::new();

	set.spawn(publish_from_stdin(client.clone()));
	if config.events.len() > 0 {
		set.spawn(consume_to_stdout(client, config.events));
	}

	while set.join_next().await.is_some() {}
	Ok(())
}
