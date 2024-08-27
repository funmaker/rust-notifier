#![feature(extract_if)]
#![feature(result_flattening)]
#![feature(trace_macros)]
#![feature(try_blocks)]
#![feature(never_type)]
#![feature(duration_constructors)]

use std::env;
use std::time::Duration;
use getopts::Options;
use futures::future;
use anyhow::Result;

mod utils;
mod feeds;
mod config;
use config::Config;
mod providers;
use providers::Providers;
mod interfaces;
use interfaces::Interfaces;
mod state;
use state::State;


#[tokio::main]
pub async fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	let mut opts = Options::new();
	
	opts.optopt("c", "config", "Select fallback device to use", "config.json");
	opts.optflag("h", "help", "Print this help menu");
	
	let matches = opts.parse(&args[1..])?;
	
	if matches.opt_present("h") {
		print_usage(&program, opts);
		return Ok(());
	}
	
	let config = matches.opt_get("c")?
	                    .unwrap_or("config.json".to_string());
	
	println!("Loading config...");
	let config = Config::load(config).await?;
	println!("Config Loaded");
	
	let mut providers = Providers::new(config.providers);
	let interfaces = Interfaces::new(config.interfaces);
	let state = State::new(config.feeds);
	let fetch_interval = Duration::from_secs(config.fetch_interval_secs);
	
	future::try_join(providers.fetch_loop(state.clone(), fetch_interval),
	                 interfaces.serve(state.clone())).await?;
	
	Ok(())
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	print!("{}", opts.usage(&brief));
}
