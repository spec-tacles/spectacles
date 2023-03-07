use std::{
	borrow::Cow,
	fmt::Debug,
	sync::{Arc, RwLock},
	time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Error, Result};
use bytes::Bytes;
use futures::{
	stream::{iter, poll_fn, repeat_with, select, select_all},
	Future, Stream, StreamExt, TryFutureExt, TryStream, TryStreamExt,
};
use nanoid::nanoid;
use redust::{
	model::stream::{
		claim::AutoclaimResponse,
		read::{Entries, Field, ReadResponse},
		Id,
	},
	pool::Pool,
	resp::from_data,
};
use spectacles::{to_vec, Value};
use tokio::time::sleep;

use self::message::Message;

pub mod message;

const DEFAULT_MAX_CHUNK: &[u8] = b"10";
const DEFAULT_BLOCK_INTERVAL: &[u8] = b"5000";
const DEFAULT_BLOCK_DURATION: Duration = Duration::from_secs(5);
const DEFAULT_MIN_IDLE_TIME: &[u8] = b"10000";
pub const STREAM_DATA_KEY: Field<'static> = Field(Cow::Borrowed(b"data"));
pub const STREAM_TIMEOUT_KEY: Field<'static> = Field(Cow::Borrowed(b"timeout_at"));

pub fn repeat_fn<F, R, O>(mut func: F) -> impl Stream<Item = O>
where
	R: Future<Output = Option<O>>,
	F: FnMut() -> R,
{
	let mut fut = Box::pin(func());

	poll_fn(move |ctx| {
		let out = fut.as_mut().poll(ctx);

		if out.is_ready() {
			fut.set(func());
		}

		out
	})
}

#[derive(Clone)]
pub struct Client {
	pub name: Bytes,
	pub group: Bytes,
	pool: Pool<String>,
	last_autoclaim: Arc<RwLock<Id>>,
}

impl Debug for Client {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RedisBroker")
			.field("name", &String::from_utf8_lossy(&self.name))
			.field("group", &String::from_utf8_lossy(&self.group))
			.finish()
	}
}

impl Client {
	pub fn new(group: impl Into<Bytes>, pool: Pool<String>) -> Self {
		let group = group.into();
		let name = nanoid!().into();

		Self {
			name,
			group,
			pool,
			last_autoclaim: Arc::default(),
		}
	}

	pub async fn publish(&self, event: impl AsRef<str>, data: &Value) -> Result<Id> {
		let mut conn = self.pool.get().await?;
		let data = conn
			.cmd([
				b"XADD".as_slice(),
				event.as_ref().as_bytes(),
				b"*",
				&STREAM_DATA_KEY.0,
				&to_vec(data)?,
			])
			.await?;

		Ok(from_data(data)?)
	}

	pub async fn publish_timeout(
		&self,
		event: impl AsRef<str>,
		data: impl AsRef<[u8]>,
		timeout: SystemTime,
	) -> Result<Id> {
		let mut conn = self.pool.get().await?;

		let timeout_bytes = timeout
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_nanos()
			.to_string()
			.into_bytes();

		let data = conn
			.cmd([
				b"XADD",
				event.as_ref().as_bytes(),
				b"*",
				&STREAM_DATA_KEY.0,
				data.as_ref(),
				&STREAM_TIMEOUT_KEY.0,
				&timeout_bytes,
			])
			.await?;

		Ok(from_data(data)?)
	}

	pub async fn ensure_events(
		&self,
		events: impl Iterator<Item = impl AsRef<[u8]>>,
	) -> Result<()> {
		let mut conn = self.pool.get().await?;

		for event in events {
			let cmd: &[&[u8]] = &[
				b"XGROUP",
				b"CREATE",
				event.as_ref(),
				&*self.group,
				b"$",
				b"MKSTREAM",
			];

			match conn.cmd(cmd).await {
				Ok(_) => (),
				Err(redust::Error::Redis(err)) if err.starts_with("BUSYGROUP") => (),
				Err(e) => return Err(e.into()),
			}
		}

		Ok(())
	}

	/// Consume events from the broker.
	pub fn consume<'s, T, U>(
		&'s self,
		events: T,
	) -> impl TryStream<Ok = Message, Error = Error> + Unpin + 's
	where
		T: AsRef<[U]> + Clone + 's,
		U: AsRef<[u8]> + 's,
	{
		let autoclaim = self.autoclaim_all(events.clone()).into_stream();
		let claim = self.claim(events).into_stream();

		select(autoclaim, claim)
	}

	fn claim<'s, T, U>(
		&'s self,
		events: T,
	) -> impl TryStream<Ok = Message, Error = Error> + Unpin + 's
	where
		T: AsRef<[U]> + Clone + 's,
		U: AsRef<[u8]>,
	{
		let fut_fn = move || {
			self.get_messages(events.clone())
				.map_ok(|msgs| iter(msgs.map(Ok)))
				.try_flatten_stream()
		};

		Box::pin(repeat_with(fut_fn).flatten())
	}

	async fn get_messages<'s, T, U>(
		&'s self,
		events: T,
	) -> Result<impl Iterator<Item = Message> + Unpin + 's>
	where
		T: AsRef<[U]>,
		U: AsRef<[u8]>,
	{
		let read = self.xreadgroup(events).await?.unwrap_or_default();

		let messages = read.0.into_iter().flat_map(move |(event, entries)| {
			entries.0.into_iter().map(move |(id, entry)| {
				Message::new(id, entry, Bytes::copy_from_slice(&event.0), self.clone())
			})
		});

		Ok(messages)
	}

	async fn xreadgroup<T, U>(&self, events: T) -> Result<Option<ReadResponse<'static>>, Error>
	where
		T: AsRef<[U]>,
		U: AsRef<[u8]>,
	{
		let events = events.as_ref();
		let ids = vec![&b">"[..]; events.len()];
		let mut cmd: Vec<&[u8]> = vec![
			b"XREADGROUP",
			b"GROUP",
			&*self.group,
			&*self.name,
			b"COUNT",
			DEFAULT_MAX_CHUNK,
			b"BLOCK",
			DEFAULT_BLOCK_INTERVAL,
			b"STREAMS",
		];
		cmd.extend(events.iter().map(|b| b.as_ref()));
		cmd.extend_from_slice(&ids);

		let data = self.pool.get().await?.cmd(cmd).await?;
		// debug!(?data);
		Ok(from_data(data)?)
	}

	async fn xautoclaim(&self, event: &[u8]) -> Result<Entries<'static>, Error> {
		let id = self.last_autoclaim.read().unwrap().to_string();

		let cmd = [
			b"XAUTOCLAIM",
			event,
			&*self.group,
			&*self.name,
			DEFAULT_MIN_IDLE_TIME,
			id.as_bytes(),
			b"COUNT",
			DEFAULT_MAX_CHUNK,
		];

		let mut conn = self.pool.get().await?;

		let data = conn.cmd(cmd).await?;
		// debug!(?data);

		let res = from_data::<AutoclaimResponse>(data)?;
		*self.last_autoclaim.write().unwrap() = res.0;
		Ok(res.1)
	}

	fn autoclaim_all<'s, T, U>(
		&'s self,
		events: T,
	) -> impl TryStream<Ok = Message, Error = Error> + 's
	where
		T: AsRef<[U]>,
		U: AsRef<[u8]>,
	{
		let streams = events
			.as_ref()
			.into_iter()
			.map(|event| {
				let event = Bytes::copy_from_slice(event.as_ref());
				move || {
					let event = event.clone();
					async move { Some(self.autoclaim_event(event).await) }
				}
			})
			.map(repeat_fn)
			.map(TryStreamExt::try_flatten);

		select_all(streams)
	}

	/// Autoclaim an event and return a stream of messages found during the autoclaim. The returned
	/// future output is always [`Some`], intended to improve ergonomics when used with
	/// [`repeat_fn`].
	///
	/// Delays every invocation of `xautoclaim` by [`DEFAULT_BLOCK_DURATION`], since `xautoclaim`
	/// does not support blocking.
	async fn autoclaim_event<'s>(
		&'s self,
		event: Bytes,
	) -> Result<impl TryStream<Ok = Message, Error = Error> + 's> {
		sleep(DEFAULT_BLOCK_DURATION).await;

		let messages = self
			.xautoclaim(&event)
			.await?
			.0
			.into_iter()
			.map(move |(id, data)| {
				Ok::<_, Error>(Message::new(id, data, event.clone(), self.clone()))
			});

		Ok(iter(messages))
	}
}
