use std::{
	collections::HashMap,
	env::args,
	io::{stdout, Write},
};

use bson::to_vec;
use fake::{Fake, Faker};
use spectacles::Event;

fn send_output(mut out: impl Write) {
	let data = Faker.fake::<HashMap<String, String>>();
	let event = Event {
		data,
		name: "test".to_string(),
	};

	out.write_all(&to_vec(&event).unwrap()).unwrap();
}

fn main() {
	let mut args = args();
	let mut out = stdout();

	match args.nth(1).map(|arg| arg.parse::<usize>().unwrap()) {
		Some(count) => {
			for _ in 0..count {
				send_output(&mut out)
			}
		}
		None => loop {
			send_output(&mut out);
		},
	}
}
