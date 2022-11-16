use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "spectacles-redis")]
pub struct Opt {
	#[structopt(long, short, default_value = "localhost:6379")]
	pub address: String,
	#[structopt(long, short)]
	pub group: String,
	#[structopt(long, short)]
	pub events: Vec<String>,
}
