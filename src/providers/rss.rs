use std::error::Error;
use async_trait::async_trait;
use futures::{StreamExt, TryFutureExt};
use rss::Channel;
use bytes::Bytes;
use chrono::DateTime;
use itertools::Itertools;

use super::Provider;
use crate::utils::{Json, Map, hash, IteratorEx};
use crate::config::ConfigFeedEntry;
use crate::feeds::{Feed, Entry};

const MAX_CON_REQUESTS: usize = 10;

pub struct RssProvider;

impl RssProvider {
	pub fn new(_config: Json) -> Result<Self, Box<dyn Error>> {
		Ok(RssProvider)
	}
}

#[async_trait(?Send)]
impl Provider for RssProvider {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>) -> Map<Feed> {
		// Url -> Feed
		let data = config.values()
		                 .map(|entry| entry.provider_data.as_str())
		                 .flatten()
		                 .unique()
		                 .into_stream()
		                 .map(|url| async move {
			                 let content = reqwest::get(url)
				                 .and_then(|res| res.bytes())
				                 .await;
			                 
			                 (url, content)
		                 })
		                 .buffer_unordered(MAX_CON_REQUESTS)
		                 .map(|(url, content)| {
			                 let feed = parse_response(content, url);
			
			                 (url.to_string(), feed)
		                 })
		                 .collect::<Map<_>>()
		                 .await;
		
		config.into_iter()
		      .map(|(name, entry)| {
			      let feed = serde_json::from_value(entry.provider_data.clone())
			                            .map(|url: String| data.get(&url).cloned().unwrap())
			                            .unwrap_or_else(|err| Feed::from_err("Failed to parse provider_data", &err.to_string()));
			      
			      (name, feed)
		      })
		      .collect()
	}
}

fn parse_response(response: reqwest::Result<Bytes>, url: &str) -> Feed {
	let parsed = response.map(|content| Channel::read_from(&*content));
	
	match parsed {
		Ok(Ok(chan)) => {
			let mut feed = Feed::new();
			
			for item in chan.items() {
				let title = item.title().clone().unwrap_or("<No Title>");
				let timestamp = item.pub_date()
				                    .and_then(|time| DateTime::parse_from_rfc2822(&time).ok());
				let guid = item.guid()
				               .map(|g| g.value().to_string())
				               .unwrap_or_else(|| hash(&(&title, &timestamp)));
				
				let entry = Entry::new(&title, &guid)
				                  .set_description(item.description().or(item.content().clone()).map(str::to_string))
				                  .set_link(item.link().map(str::to_string))
				                  .set_timestamp(timestamp);
				
				feed.notifications.push(entry);
			}
			
			feed
		},
		Ok(Err(err)) => Feed::from_err(&format!("Unable to parse {}", url), &err.to_string()),
		Err(err) => Feed::from_err(&format!("Unable to fetch {}", url), &err.to_string()),
	}
}
