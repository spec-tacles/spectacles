use std::io::stdin;

use bson::{from_reader, to_vec};
use options::Opt;
use paho_mqtt::{Client, ConnectOptions, CreateOptions, Message};
use spectacles::AnyEvent;
use structopt::StructOpt;

mod options;

fn main() -> anyhow::Result<()> {
	let opt = Opt::from_args();

	let client_options = CreateOptions::from(opt.create);
	let mqtt = Client::new(client_options).unwrap();

	let connect_options = ConnectOptions::try_from(opt.connect)?;
	mqtt.connect(connect_options)?;

	let mut in_ = stdin();

	loop {
		let event = from_reader::<_, AnyEvent>(&mut in_)?;
		let event_bytes = to_vec(&event.data)?;

		let message = Message::new(event.name, event_bytes, 2);
		mqtt.publish(message)?;
	}
}
