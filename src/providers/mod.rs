use std::time::{Duration, Instant};
use std::sync::Arc;
use async_trait::async_trait;
use futures::future::join_all;
use err_derive::Error;
use serde::Deserialize;
use tokio::task::JoinError;
use tokio::time;

mod null;
use null::NullProvider;
mod rss;
use self::rss::RssProvider;
mod youtube;
use youtube::YouTubeProvider;

use crate::config::ConfigFeedEntry;
use crate::feeds::{Feed, Feeds};
use crate::utils::{Map, Json};
use crate::state::State;

#[async_trait(?Send)]
trait Provider: Send {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>) -> Map<Feed>;
}

pub struct Providers {
	providers: Map<Box<dyn Provider>>,
}

fn boxed<P: Provider + 'static>(result: Result<P, Box<dyn std::error::Error>>) -> Result<Box<dyn Provider>, ProviderError> {
	match result {
		Ok(provider) => Ok(Box::new(provider)),
		Err(err) => Err(ProviderError::InitError(err)),
	}
}

#[derive(Deserialize)]
struct AnyProviderConfig {
	enabled: bool
}

fn init_provider(name: String, config: Json) -> Box<dyn Provider> {
	match serde_json::from_value(config.clone()) {
		Ok(AnyProviderConfig{ enabled }) if !enabled => return Box::new(NullProvider::new(Box::new(ProviderError::Disabled))),
		Err(err) => return Box::new(NullProvider::new(Box::new(err))),
		_ => {},
	}
	
	// Add new providers here
	let provider = match &*name {
		"rss" => boxed(RssProvider::new(config)),
		"youtube" => boxed(YouTubeProvider::new(config)),
		_ => Err(ProviderError::NotFound),
	};
	
	provider.unwrap_or_else(|err| {
		eprintln!("Unable to load {} provider: {}", name, err.to_string());
		Box::new(NullProvider::new(Box::new(err)))
	})
}

impl Providers {
	pub fn new(configs: Map<Json>) -> Self {
		let providers = configs.into_iter()
		                       .map(|(name, config)| (name.clone(), init_provider(name, config)))
		                       .collect();
		
		Providers { providers }
	}
	
	pub async fn fetch_feeds(&mut self, feeds_configs: &Map<ConfigFeedEntry>) -> Feeds {
		let feeds = self.providers.iter_mut()
		                          .map(|(name, provider)| async move {
			                          let configs: Map<&ConfigFeedEntry> = feeds_configs.iter()
			                                                                            .filter(|entry| &entry.1.provider == name)
			                                                                            .map(|(name, entry)| (name.clone(), entry))
			                                                                            .collect();
			                          
			                          let mut feeds = provider.fetch(configs).await;
			                          
			                          for (name, feed) in feeds.iter_mut() {
				                          if let Some(config) = feeds_configs.get(name) {
					                          for entry in feed.iter_mut() {
						                          entry.feed_name = entry.feed_name.take().or(Some(name.clone()));
						                          entry.color = entry.color.take().or(config.color.clone());
					                          }
				                          }
			                          }
			                          
			                          feeds
		                          });
		
		join_all(feeds).await
		               .into_iter()
		               .fold(Feeds::new(), |mut acc, mut feeds| { acc.append(&mut feeds); acc })
	}
	
	pub async fn fetch_loop(&mut self, state: State, fetch_interval: Duration) -> Result<(), JoinError> {
		let mut interval = time::interval(fetch_interval);
		
		loop {
			interval.tick().await;
			
			println!("Fetching feeds...");
			let now = Instant::now();
			
			let feeds = self.fetch_feeds(&state.feed_entries.load()).await;
			state.feeds.store(Arc::new(feeds));
			
			println!("Fetch done. ({}s)", (now.elapsed().as_secs_f32() * 100.0).round() / 100.0);
		}
	}
}

#[derive(Debug, Error)]
pub enum ProviderError {
	#[error(display = "Provider not found")] NotFound,
	#[error(display = "Provider is disabled")] Disabled,
	#[error(display = "Provider failed to init: {}", _0)] InitError(Box<dyn std::error::Error>),
}
