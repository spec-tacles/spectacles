use std::io::{stdin, stdout, Write};

use bson::{from_reader, Bson};
use serde_json::to_writer;

fn main() -> anyhow::Result<()> {
	let mut in_ = stdin();
	let mut out = stdout();

	loop {
		let data = from_reader::<_, Bson>(&mut in_)?;
		to_writer(&mut out, &data)?;
		out.flush()?;
	}
}
