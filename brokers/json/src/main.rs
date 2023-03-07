use std::io::{stdin, stdout, Write};

use serde_json::to_writer;
use spectacles::{from_read, Value};

fn main() -> anyhow::Result<()> {
	let mut in_ = stdin();
	let mut out = stdout();

	loop {
		let data = from_read::<_, Value>(&mut in_)?;
		to_writer(&mut out, &data)?;
		out.flush()?;
	}
}
