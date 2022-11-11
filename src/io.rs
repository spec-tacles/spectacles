use std::io::stdin;

use bson::{de::Error, from_reader};
use futures::TryStream;
use serde::de::DeserializeOwned;
use tokio::{sync::mpsc, task::spawn_blocking};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub fn read<T>() -> impl TryStream<Ok = T, Error = Error>
where
	T: DeserializeOwned + Send + Sync + 'static,
{
	let (tx, rx) = mpsc::unbounded_channel();

	spawn_blocking(move || {
		let mut in_ = stdin();
		loop {
			if tx.send(from_reader::<_, T>(&mut in_)).is_err() {
				break;
			}
		}
	});

	UnboundedReceiverStream::new(rx)
}
