use std::io::{stdin, stdout, Write};

use serde_transcode::transcode;

fn main() -> anyhow::Result<()> {
	let mut stdin = stdin().lock();
	let mut stdout = stdout().lock();

	loop {
		let mut de = rmp_serde::Deserializer::new(&mut stdin);
		let mut ser = serde_json::Serializer::new(&mut stdout);
		transcode(&mut de, &mut ser)?;
		stdout.flush()?;
	}
}
