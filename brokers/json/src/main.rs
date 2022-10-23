use std::io::{stdout, Write};

use flexbuffers::Reader;
use futures::StreamExt;
use serde_json::Serializer;
use serde_transcode::transcode;
use tokio::io::stdin;
use tokio_util::io::ReaderStream;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	let mut stdin = ReaderStream::new(stdin());
	let mut stdout = stdout();

	while let Some(buf) = stdin.next().await {
		let buf = buf?;
		let de = Reader::get_root(&*buf)?;
		let mut ser = Serializer::new(&mut stdout);
		transcode(de, &mut ser)?;
		stdout.flush()?;
	}

	Ok(())
}
