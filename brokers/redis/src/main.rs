use anyhow::Result;
use bson::to_vec;
use futures::{StreamExt, TryStreamExt};
use redust::pool::{Manager, Pool};
use spectacles::{io::read, AnyEvent, EventRef};
use tokio::{
	io::{stdout, AsyncWriteExt},
	task::JoinSet,
};

use crate::client::Client;

mod client;

async fn publish_from_stdin(client: Client) -> Result<()> {
	let mut stream = read::<AnyEvent>();
	while let Some(event) = stream.next().await {
		client.publish(event.name, &event.data).await?;
	}

	Ok(())
}

async fn consume_to_stdout(client: Client) -> Result<()> {
	let mut out = stdout();
	let mut stream = client.consume(["events"]);
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
	let manager = Manager::new("".to_string());
	let pool = Pool::builder(manager).build()?;
	let client = Client::new("group", pool);

	client.ensure_events(["events"].into_iter()).await?;

	let mut set = JoinSet::new();

	set.spawn(publish_from_stdin(client.clone()));
	set.spawn(consume_to_stdout(client));

	while set.join_next().await.is_some() {}
	Ok(())
}
