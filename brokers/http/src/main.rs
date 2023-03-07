use std::io::{stdout, Write};

use axum::{extract::Path, routing::on, Router, Server};
use bytes::Bytes;
use futures::StreamExt;
use options::Opt;
use reqwest::{Client, StatusCode};
use spectacles::{from_slice, init_tracing, io::read, to_vec, AnyEvent, EventRef, Value};
use structopt::StructOpt;
use tokio::task::JoinSet;
use tracing::{debug, info, info_span, warn, Instrument};

mod options;

async fn handle_http_out(opt: Opt) -> anyhow::Result<()> {
	let client = Client::new();
	let mut rd = read::<AnyEvent>();

	let mut set = JoinSet::new();

	while let Some(event) = rd.next().await {
		let data = to_vec(&event.data)?;
		let client = client.clone();
		let opt = opt.clone();
		let span = info_span!("make_request", ?event);

		set.spawn(
			async move {
				let result = client
					.request(opt.method, format!("{}{}", opt.url, event.name))
					.body(data)
					.send()
					.await;

				match result {
					Ok(response) => debug!(?response),
					Err(err) => warn!(%err),
				}
			}
			.instrument(span),
		);
	}

	while set.join_next().await.is_some() {}
	Ok(())
}

async fn handle_http_in(opt: Opt) -> anyhow::Result<()> {
	async fn handle_request(
		Path(path): Path<String>,
		body: Bytes,
	) -> Result<StatusCode, StatusCode> {
		let data = from_slice::<Value>(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
		let event = EventRef { data, name: &path };
		debug!(?event);

		let bytes = to_vec(&event).unwrap();
		stdout().write_all(&bytes).unwrap();

		Ok(StatusCode::NO_CONTENT)
	}

	let app = Router::new().route(
		&format!("{}/:name", opt.url.path()),
		on(opt.method.try_into().unwrap(), handle_request),
	);

	info!("Listening on {}", opt.url);
	Server::bind(&opt.url.socket_addrs(|| None).unwrap()[0])
		.serve(app.into_make_service())
		.await
		.unwrap();

	Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	init_tracing();

	let opt = Opt::from_args();
	info!(?opt);

	let mut set = JoinSet::new();
	if !opt.r#in || opt.out {
		set.spawn(handle_http_out(opt.clone()));
	}

	if opt.r#in {
		set.spawn(handle_http_in(opt));
	}

	while set.join_next().await.is_some() {}
	Ok(())
}
