use std::collections::HashSet;
use std::error::Error;
use async_trait::async_trait;
use futures::{stream, StreamExt, TryFutureExt};
use rss::Channel;
use bytes::Bytes;
use chrono::DateTime;

use super::Provider;
use crate::utils::{Json, Map, hash};
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
		let urls = config.values()
		                 .filter_map(move |entry| entry.provider_data.as_str())
		                 .map(|s| s.to_string())
		                 .collect::<HashSet<_>>();
		
		let data = stream::iter(urls)
		                  .map(|url| async move {
			                  let content = reqwest::get(&url)
				                                    .and_then(|res| res.bytes())
				                                    .await;
			                  
			                  (url, content)
		                  })
		                  .buffer_unordered(MAX_CON_REQUESTS)
		                  .map(|(url, content)| {
			                  let feed = resp_to_feed(content, &url);
			                  
			                  (url, feed)
		                  })
		                  .collect::<Map<Feed>>()
		                  .await;
		
		config.into_iter()
		      .map(|(name, entry)|
			      (name, entry.provider_data
			                  .as_str()
			                  .map(|url| data.get(url))
			                  .flatten()
			                  .cloned()
			                  .unwrap_or_else(|| Feed::from_err("Unable to parse provider_data", &format!("Expected String, got {}", entry.provider_data.to_string())))))
		      .collect()
	}
}

fn resp_to_feed(response: reqwest::Result<Bytes>, url: &str) -> Feed {
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
