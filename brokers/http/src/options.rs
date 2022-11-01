use reqwest::{Method, Url};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub struct Opt {
	#[structopt(long, short, default_value = "POST")]
	pub method: Method,
	#[structopt()]
	pub url: Url,
	#[structopt(long)]
	pub r#in: bool,
	#[structopt(long)]
	pub out: bool,
}
