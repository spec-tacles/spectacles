use std::io::stdin;

use bson::{from_reader, to_vec};
use paho_mqtt::{Client, Message};
use spectacles::AnyEvent;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "spectacles-mqtt")]
struct Opt {
	#[structopt(long)]
	url: String,
}

fn main() -> anyhow::Result<()> {
	let opt = Opt::from_args();

	let mqtt = Client::new(opt.url).unwrap();
	mqtt.connect(None)?;

	let mut in_ = stdin();

	loop {
		let event = from_reader::<_, AnyEvent>(&mut in_)?;
		let event_bytes = to_vec(&event.data)?;

		let message = Message::new(event.name, event_bytes, 2);
		mqtt.publish(message)?;
	}
}
