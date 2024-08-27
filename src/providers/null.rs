use anyhow::Error;
use async_trait::async_trait;

use super::Provider;
use crate::utils::Map;
use crate::config::ConfigFeedEntry;
use crate::feeds::Feed;

pub struct NullProvider {
	error: Error,
}

impl NullProvider {
	pub fn new(error: Error) -> Self {
		NullProvider{
			error,
		}
	}
}

#[async_trait(?Send)]
impl Provider for NullProvider {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>, _client: reqwest::Client) -> Map<Feed> {
		let feed = Feed::from_err("Failed to load provider", &self.error);
		
		config.into_iter()
		      .map(|(name, _)| (name, feed.clone()))
		      .collect()
	}
}
