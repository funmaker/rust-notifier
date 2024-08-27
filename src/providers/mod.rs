use std::time::{Duration, Instant};
use std::sync::Arc;
use async_trait::async_trait;
use futures::future::join_all;
use anyhow::Result;
use thiserror::Error;
use serde::Deserialize;
use tokio::time;

mod null;
mod rss;
mod youtube;
mod chan;
mod vinesauce;

use null::NullProvider;
use youtube::YouTubeProvider;
use chan::ChanProvider;
use self::rss::RssProvider;
use crate::config::ConfigFeedEntry;
use crate::feeds::{Feed, Feeds};
use crate::utils::{Map, Json};
use crate::state::State;
use crate::providers::vinesauce::VinesauceProvider;

#[async_trait(?Send)]
trait Provider: Send {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>, client: reqwest::Client) -> Map<Feed>;
}

pub struct Providers {
	providers: Map<Box<dyn Provider>>,
}

fn boxed<P: Provider + 'static>(result: Result<P>) -> Result<Box<dyn Provider>> {
	match result {
		Ok(provider) => Ok(Box::new(provider)),
		Err(err) => Err(err),
	}
}

#[derive(Deserialize)]
struct AnyProviderConfig {
	enabled: bool
}

fn init_provider(name: String, config: Json) -> Box<dyn Provider> {
	match serde_json::from_value(config.clone()) {
		Ok(AnyProviderConfig{ enabled }) if !enabled => return Box::new(NullProvider::new(ProviderDisabledError.into())),
		Err(err) => return Box::new(NullProvider::new(err.into())),
		_ => {},
	}
	
	// Add new providers here
	let provider = match &*name {
		"rss" => boxed(RssProvider::new(config)),
		"youtube" => boxed(YouTubeProvider::new(config)),
		"chan" => boxed(ChanProvider::new(config)),
		"vinesauce" => boxed(VinesauceProvider::new(config)),
		_ => Err(ProviderNotFoundError.into()),
	};
	
	provider.unwrap_or_else(|err| {
		eprintln!("Unable to load {} provider: {}", name, err.to_string());
		Box::new(NullProvider::new(err.into()))
	})
}

impl Providers {
	pub fn new(configs: Map<Json>) -> Self {
		let providers = configs.into_iter()
		                       .map(|(name, config)| (name.clone(), init_provider(name, config)))
		                       .collect();
		
		Providers { providers }
	}
	
	pub async fn fetch_feeds(&mut self, feeds_configs: &Map<ConfigFeedEntry>, client: reqwest::Client) -> Feeds {
		let client_ref = &client;
		let feeds = self.providers.iter_mut()
		                          .map(|(name, provider)| async move {
			                          let client = client_ref.clone();
			                          
			                          let configs: Map<&ConfigFeedEntry> = feeds_configs.iter()
			                                                                            .filter(|entry| &entry.1.provider == name)
			                                                                            .map(|(name, entry)| (name.clone(), entry))
			                                                                            .collect();
			                          
			                          let mut feeds = provider.fetch(configs, client.clone()).await;
			                          
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
	
	pub async fn fetch_loop(&mut self, state: State, fetch_interval: Duration) -> Result<()> {
		let mut interval = time::interval(fetch_interval);
		let client = reqwest::Client::new();
		
		loop {
			interval.tick().await;
			
			println!("Fetching feeds...");
			let now = Instant::now();
			
			let feeds = self.fetch_feeds(&state.feed_entries.load(), client.clone()).await;
			state.feeds.store(Arc::new(feeds));
			
			println!("Fetch done. ({}s)", (now.elapsed().as_secs_f32() * 100.0).round() / 100.0);
		}
	}
}

#[derive(Debug, Error)]
#[error("Provider not found")]
pub struct ProviderNotFoundError;

#[derive(Debug, Error)]
#[error("Provider is disabled")]
pub struct ProviderDisabledError;
