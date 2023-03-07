use anyhow::Result;
use paho_mqtt::{Client, ConnectOptions, CreateOptions, Message};
use spectacles::{from_read, from_slice, init_tracing, to_vec, to_writer, AnyEvent};
use std::{
	io::{stdin, stdout},
	thread::spawn,
};

use crate::config::Config;

mod config;

fn publish_from_stdin(mqtt: Client, qos: i32) -> Result<()> {
	let mut in_ = stdin();

	loop {
		let event = from_read::<_, AnyEvent>(&mut in_)?;

		let message = Message::new(event.name, to_vec(&event.data)?, qos);
		mqtt.publish(message)?;
	}
}

fn consume_to_stdout(mqtt: Client, events: Vec<String>, qos: i32) -> Result<()> {
	let mut out = stdout();

	mqtt.subscribe_many(&events, &vec![qos; events.len()])?;
	let stream = mqtt.start_consuming();

	while let Some(message) = stream.recv()? {
		let event = AnyEvent {
			name: message.topic().to_owned(),
			data: from_slice(message.payload())?,
		};
		to_writer(&mut out, &event)?;
	}

	todo!()
}

fn main() -> Result<()> {
	init_tracing();

	let config = Config::build()?;

	let client_options = CreateOptions::from(config.create);
	let mqtt = Client::new(client_options).unwrap();

	let connect_options = ConnectOptions::try_from(config.connect)?;
	mqtt.connect(connect_options)?;

	let mqtt_pub = mqtt.clone();
	let handle = spawn(move || publish_from_stdin(mqtt_pub, config.qos));

	if !config.events.is_empty() {
		consume_to_stdout(mqtt, config.events, config.qos)?;
	}

	handle.join().unwrap()?;
	Ok(())
}
