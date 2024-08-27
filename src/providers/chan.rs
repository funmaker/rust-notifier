use std::cell::Cell;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time;
use tokio::time::{Duration, Instant};
use itertools::Itertools;
use futures::{StreamExt, TryFutureExt};
use regex::RegexBuilder;
use tokio_stream::wrappers::IntervalStream;
use anyhow::Result;
use reqwest::header;
use thiserror::Error;
use crate::utils::{Map, Json, IteratorEx, hash};
use crate::feeds::{Feed, Entry};
use crate::config::ConfigFeedEntry;
use super::Provider;

const MAX_CON_REQUESTS: usize = 1;

pub struct ChanProvider;

impl ChanProvider {
	pub fn new(_config: Json) -> Result<Self> {
		Ok(ChanProvider)
	}
}

#[derive(Deserialize)]
struct ProviderData {
	boards: Vec<String>,
	filter: String,
}

type Catalog = Vec<Page>;

#[derive(Deserialize, Debug)]
struct Page {
	page: i32,
	threads: Vec<OP>,
}

#[derive(Deserialize, Debug)]
struct OP {
	no: i32,
	time: i64,
	sub: Option<String>,
	com: Option<String>,
	tim: Option<u64>,
	replies: i32,
	images: i32,
	semantic_url: String,
}

#[derive(Serialize)]
struct Extra {
	replies: i32,
	images: i32,
	page: i32,
	board: String,
	id: i32,
}

thread_local! {
    pub static LAST_FETCH: Cell<DateTime<Utc>> = Cell::new(Utc::now() - Duration::from_hours(1));
}

#[async_trait(?Send)]
impl Provider for ChanProvider {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>, client: reqwest::Client) -> Map<Feed> {
		let last_fetch = LAST_FETCH.replace(Utc::now());
		
		let client_ref = &client;
		let catalogs = config.values()
		                     .flat_map(|config| serde_json::from_value::<ProviderData>(config.provider_data.clone()))
		                     .flat_map(|provider_data| provider_data.boards.into_iter())
		                     .unique()
		                     .into_stream()
		                     .zip(IntervalStream::new(time::interval(Duration::from_secs(1)))) // 4chan api should be called in 1s intervals
		                     .map(|(board, time)| async move {
			                     let client = client_ref.clone();
			                     let url = format!("http://a.4cdn.org/{}/catalog.json", board);
			                     
			                     let content: Result<_> = try {
				                     let bytes = client.clone()
				                                       .get(&url)
				                                       .header(header::IF_MODIFIED_SINCE, last_fetch.to_rfc2822())
				                                       .header(header::USER_AGENT, format!("rust-notifier/{}", option_env!("CARGO_PKG_VERSION").unwrap_or("0.0.0")))
				                                       .send()
				                                       .await?
				                                       .error_for_status()?
				                                       .bytes()
				                                       .await?;
				                     serde_json::from_slice::<Catalog>(&*bytes)?
			                     };
			                     
			                     (board, content)
		                     })
		                     .buffer_unordered(MAX_CON_REQUESTS)
		                     .collect::<Map<_>>()
		                     .await;
		
		config.into_iter()
		      .map(|(name, config)| {
			      let provider_data: ProviderData = match serde_json::from_value(config.provider_data.clone()) {
				      Ok(provider_data) => provider_data,
				      Err(err) => return (name, Feed::from_err("Unable to parse providerData", &err.into())),
			      };
			      
			      let mut feed = Feed::new();
			      let filter = RegexBuilder::new(&provider_data.filter)
			                                .size_limit(1024 * 32)
			                                .dfa_size_limit(1024 * 32)
			                                .nest_limit(10)
			                                .case_insensitive(true)
			                                .build();
			      
			      let filter = match filter {
				      Ok(filter) => filter,
				      Err(err) => return (name, Feed::from_err("Unable to parse filter", &err.into())),
			      };
			      
			      for board in provider_data.boards {
				      match catalogs.get(&board) {
					      Some(Ok(catalog)) => {
						      catalog.iter()
						             .flat_map(|page| page.threads.iter()
						                                  .filter(|op|
							                                  op.sub.as_ref().map_or(false, |sub| filter.is_match(sub)) ||
							                                  op.com.as_ref().map_or(false, |com| filter.is_match(com))
						                                  )
						                                  .map(move |op| (page.page, op)))
						             .map(|(page, op)| {
							             Entry::new(&op.sub.as_ref().unwrap_or(&op.semantic_url.replace("-", " ")), &hash(&(op.no, &board)))
							                   .set_description(op.com.clone())
							                   .link(&format!("https://boards.4chan.org/{}/thread/{}", board, op.no))
							                   .set_timestamp(DateTime::from_timestamp(op.time, 0))
							                   .set_image_url(op.tim.map(|tim| format!("https://i.4cdn.org/{}/{}s.jpg", board, tim)))
							                   .set_extra(serde_json::to_value(Extra {
								                   replies: op.replies,
								                   images: op.images,
								                   page,
								                   board: board.clone(),
								                   id: op.no
							                   }).ok())
						             })
						             .for_each(|op| feed.status.push(op));
					      },
					      Some(Err(err)) => feed.add_err(&format!("Unable to fetch board {}", board), err),
					      None => feed.add_err(&format!("Unable to fetch board {}", board), &BoardNotFound.into()),
				      }
			      }
			      
			      feed.status.sort();
			      
			      (name, feed)
		      })
		      .collect()
	}
}

#[derive(Debug, Copy, Clone, Error)]
#[error("Board Not Found")]
pub struct BoardNotFound;
