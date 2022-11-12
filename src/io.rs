use std::io::{stdin, ErrorKind};

use bson::{de::Error, from_reader};
use futures::Stream;
use serde::de::DeserializeOwned;
use tokio::{sync::mpsc, task::spawn_blocking};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::warn;

pub fn read<T>() -> impl Stream<Item = T>
where
	T: DeserializeOwned + Send + Sync + 'static,
{
	let (tx, rx) = mpsc::unbounded_channel();

	spawn_blocking(move || {
		let mut in_ = stdin();
		loop {
			match from_reader::<_, T>(&mut in_) {
				Ok(data) => {
					if tx.send(data).is_err() {
						warn!("Read value from STDIN but receiver is closed to receive it");
						break;
					}
				}
				Err(Error::Io(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
				Err(err) => warn!(%err),
			}
		}
	});

	UnboundedReceiverStream::new(rx)
}
