use std::{
	io::{stdin, stdout, Write},
	thread::spawn,
};

use anyhow::Result;
use bson::{from_reader, to_vec};
use options::Opt;
use paho_mqtt::{Client, ConnectOptions, CreateOptions, Message};
use spectacles::{init_tracing, AnyEvent, EventRef};
use structopt::StructOpt;

mod options;

fn publish_from_stdin(mqtt: Client, qos: i32) -> Result<()> {
	let mut in_ = stdin();

	loop {
		let event = from_reader::<_, AnyEvent>(&mut in_)?;

		let message = Message::new(event.name, to_vec(&event.data)?, qos);
		mqtt.publish(message)?;
	}
}

fn consume_to_stdout(mqtt: Client, events: Vec<String>, qos: i32) -> Result<()> {
	let mut out = stdout();

	mqtt.subscribe_many(&events, &vec![qos; events.len()])?;
	let stream = mqtt.start_consuming();

	while let Some(message) = stream.recv()? {
		let event = EventRef {
			name: message.topic(),
			data: message.payload(),
		};
		out.write_all(&to_vec(&event)?)?;
	}

	todo!()
}

fn main() -> Result<()> {
	init_tracing();

	let opt = Opt::from_args();

	let client_options = CreateOptions::from(opt.create);
	let mqtt = Client::new(client_options).unwrap();

	let connect_options = ConnectOptions::try_from(opt.connect)?;
	mqtt.connect(connect_options)?;

	let mqtt_pub = mqtt.clone();
	let handle = spawn(move || publish_from_stdin(mqtt_pub, opt.qos));

	if !opt.events.is_empty() {
		consume_to_stdout(mqtt, opt.events, opt.qos)?;
	}

	handle.join().unwrap()?;
	Ok(())
}
