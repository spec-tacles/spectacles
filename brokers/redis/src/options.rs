use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "spectacles-redis")]
pub struct Opt {
	#[structopt(long, short, default_value = "localhost:6379", env = "REDIS_ADDRESS")]
	pub address: String,
	#[structopt(long, short, env = "REDIS_GROUP")]
	pub group: String,
	#[structopt(long, short, env = "REDIS_EVENTS")]
	pub events: Vec<String>,
}
