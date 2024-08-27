use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use anyhow::Result;

use crate::utils::*;

#[derive(Deserialize, Serialize)]
pub struct Config {
	pub feeds: Map<ConfigFeedEntry>,
	pub providers: Map<Json>,
	pub interfaces: Map<Json>,
	#[serde(rename="fetchIntervalSecs")]
	pub fetch_interval_secs: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConfigFeedEntry {
	pub provider: String,
	#[serde(rename="providerData")]
	pub provider_data: Json,
	pub color: Option<String>
}

impl Config {
	pub async fn load(path: impl AsRef<Path>) -> Result<Config> {
		match File::open(&path).await {
			Ok(mut file) => {
				let mut config_content = vec![];
				file.read_to_end(&mut config_content).await?;
				
				let config: Config = serde_json::from_slice(&config_content)?;
				Ok(config)
			},
			Err(_) => {
				println!("{} not found.\nGenerating new from config_example.json.", path.as_ref().to_string_lossy());
				
				let mut example = File::open("config_example.json").await?;
				let mut example_content = vec![];
				example.read_to_end(&mut example_content).await?;
				
				let config: Config = serde_json::from_slice(&example_content)?;
				config.save(&path).await?;
				Ok(config)
			},
		}
	}
	
	pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
		let mut file = File::create(&path).await?;
		file.write_all(serde_json::to_string_pretty(self)?.as_bytes()).await?;
		Ok(())
	}
}
