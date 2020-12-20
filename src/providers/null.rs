use std::error::Error;
use async_trait::async_trait;

use super::Provider;
use crate::utils::Map;
use crate::config::ConfigFeedEntry;
use crate::feeds::Feed;

pub struct NullProvider {
	error: String,
}

impl NullProvider {
	pub fn new(error: Box<dyn Error>) -> Self {
		NullProvider{
			error: error.to_string()
		}
	}
}

#[async_trait(?Send)]
impl Provider for NullProvider {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>) -> Map<Feed> {
		let feed = Feed::from_err("Failed to load provider", &self.error);
		
		config.into_iter()
		      .map(|(name, _)| (name, feed.clone()))
		      .collect()
	}
}
