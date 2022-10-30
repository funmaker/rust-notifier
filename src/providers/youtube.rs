use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};
use err_derive::Error;
use async_trait::async_trait;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use futures::StreamExt;
use itertools::Itertools;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use chrono::DateTime;
use serde_json::json;

use crate::utils::{Json, Map, IteratorEx};
use crate::providers::Provider;
use crate::config::ConfigFeedEntry;
use crate::feeds::{Feed, Entry};

const MAX_CON_REQUESTS: usize = 10;

pub struct YouTubeProvider {
	api_key: String,
	access_token: Option<String>,
	access_expires: Instant,
	refresh_token: Option<String>,
	client_id: Option<String>,
	client_secret: Option<String>,
}

#[derive(Deserialize)]
struct YouTubeConfig {
	#[serde(rename="apiKey")]
	api_key: String,
	#[serde(rename="refreshToken")]
	refresh_token: Option<String>,
	#[serde(rename="clientId")]
	client_id: Option<String>,
	#[serde(rename="clientSecret")]
	client_secret: Option<String>,
}

impl YouTubeProvider {
	pub fn new(config: Json) -> Result<Self, Box<dyn Error>> {
		let config: YouTubeConfig = serde_json::from_value(config)?;
		
		Ok(YouTubeProvider {
			api_key: config.api_key,
			access_token: None,
			access_expires: Instant::now(),
			refresh_token: config.refresh_token,
			client_id: config.client_id,
			client_secret: config.client_secret,
		})
	}
	
	pub async fn refresh_access_token(&mut self) -> Result<String, FetchError> {
		if self.access_token.is_some() && self.access_expires > Instant::now() {
			return Ok(self.access_token.clone().unwrap())
		}
		
		let client_id = self.client_id.clone().ok_or(FetchError::NoOauth)?;
		let client_secret = self.client_secret.clone().ok_or(FetchError::NoOauth)?;
		let refresh_token = self.refresh_token.clone().ok_or(FetchError::NoOauth)?;
		
		let body = format!("client_id={}&\
							client_secret={}&\
							refresh_token={}&\
							grant_type=refresh_token", client_id, client_secret, refresh_token);
		
		let client = reqwest::Client::new();
		let result = client.post("https://oauth2.googleapis.com/token")
		                   .body(body)
		                   .header("Content-Type", "application/x-www-form-urlencoded")
		                   .send()
		                   .await?
		                   .error_for_status()?
		                   .bytes()
		                   .await?;
		
		let result = serde_json::from_slice::<GoogleOauthResponse>(&*result)?;
		
		self.access_expires = Instant::now() + Duration::from_secs(result.expires_in);
		self.access_token = Some(result.access_token.clone());
		
		Ok(result.access_token)
	}
}

fn encode<'a>(text: &'a str) -> impl Display + 'a {
	utf8_percent_encode(text, NON_ALPHANUMERIC)
}

async fn fetch<Item: DeserializeOwned + std::fmt::Debug>(url: String) -> Result<Vec<Result<Item, FetchError>>, FetchError> {
	let result = reqwest::get(&url)
	                     .await?
	                     .error_for_status()?
	                     .bytes()
	                     .await?;
	
	let result = serde_json::from_slice::<YouTubeResponse>(&*result)?;
	
	for r in result.items.iter() {
		let w = serde_json::from_value::<Item>(r.clone());
		if w.is_err() {
			eprintln!("Couldn't parse {} {:?}", serde_json::to_string_pretty(r).unwrap(), w);
		}
	}
	
	Ok(result.items.into_iter()
	               .map(serde_json::from_value)
	               .map(|r| r.map_err(Into::into))
	               .collect())
}

async fn fetch_all<Item: DeserializeOwned>(url: String) -> Result<Vec<Result<Item, FetchError>>, FetchError> {
	let mut ret = vec![];
	let mut next_page_token: Option<String> = None;
	
	loop {
		let url = next_page_token.map(|token| format!("{}&pageToken={}", &url, encode(&token)))
		                         .unwrap_or(url.to_string());
		
		let result = reqwest::get(&url)
		                           .await?
		                           .error_for_status()?
		                           .bytes()
		                           .await?;
		
		let result = serde_json::from_slice::<YouTubeResponse>(&*result)?;
		
		ret.extend(result.items.into_iter()
		                       .map(serde_json::from_value)
		                       .map(|r| r.map_err(Into::into)));
		next_page_token = result.next_page_token;
		
		if next_page_token.is_none() { break }
	}
	
	Ok(ret)
}

macro_rules! ytcall(
	{ $api_key:expr, $command:expr, $item_type:ty, $($key:expr => $value:expr),+ } => { ytcall!($api_key, $command, $item_type, $($key => $value),+; fetch_all ) };
	{ $api_key:expr, $command:expr, $item_type:ty, $($key:expr => $value:expr),+; Single } => { ytcall!($api_key, $command, $item_type, $($key => $value),+; fetch ) };
	{ $api_key:expr, $command:expr, $item_type:ty, $($key:expr => $value:expr),+; $fetch_fn:ident } => {
		{
			let mut s = format!("https://www.googleapis.com/youtube/v3/{}?key={}", $command, $api_key);
			$(
				s += &format!("&{}={}", encode($key), encode(&$value.to_string()));
			)*
			
			$fetch_fn::<$item_type>(s)
		}
	};
);

macro_rules! try_feed (
	{ $result:expr, $feed:expr, $reason:literal, $( $arg:expr ),*; $then:expr } => {
		match $result {
			Err(err) => {
				$feed.add_err(&format!( $reason, $( $arg ),* ), &err.to_string());
				$then
			},
			Ok(val) => val,
		}
	}
);

#[async_trait(?Send)]
impl Provider for YouTubeProvider {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>) -> Map<Feed> {
		let mut skipped_errors = vec![];
		
		// Feed -> Channel
		let feed_channel = config.iter()
		                         .map(|(name, entry)|
			                         (name.to_string(), serde_json::from_value::<String>(entry.provider_data.clone())
			                                                                   .map_err(Into::into))
		                         )
		                         .collect::<Map<Result<String, FetchError>>>();
		
		let uses_oauth = feed_channel.values().any(|channel| channel.as_deref().ok() == Some("mine"));
		
		let access_token = if uses_oauth {
			match self.refresh_access_token().await {
				Ok(access_token) => Ok(access_token),
				Err(err) => {
					skipped_errors.push(err);
					Err(FetchError::NoOauth)
				}
			}
		} else {
			Err(FetchError::NoOauth)
		};
		
		// Channel -> Subscriptions
		let channel_subs = feed_channel.values()
		                               .flatten()
		                               .unique()
		                               .cloned()
		                               .into_stream()
		                               .map(|channel| async {
			                               let subs = if channel == "mine" {
				                               match &access_token {
					                               Ok(access_token) => {
						                               ytcall![self.api_key, "subscriptions", YouTubeSubscription,
						                                       "part" => "snippet",
						                                       "access_token" => access_token,
						                                       "mine" => true,
						                                       "maxResults" => 50 ].await
					                               },
					                               _ => Err(FetchError::NoOauth),
				                               }
			                               } else {
				                               ytcall![self.api_key, "subscriptions", YouTubeSubscription,
				                                       "part" => "snippet",
				                                       "channelId" => channel,
				                                       "maxResults" => 50 ].await
			                               };
			                               
			                               (channel, subs)
		                               })
		                               .buffer_unordered(MAX_CON_REQUESTS)
		                               .collect::<Map<_>>()
		                               .await;
		
		// Channel -> Uploads
		let channel_uploads = channel_subs.values()
		                                  .flatten() // D
		                                  .flatten() // F
		                                  .flatten() // C
		                                  .map(|sub| sub.snippet.resource_id.channel_id.to_string())
		                                  .into_stream()
		                                  .chunks(50)
		                                  .map(|ids| async {
			                                  let channels = ytcall![self.api_key, "channels", YouTubeChannel,
			                                                         "part" => "contentDetails",
			                                                         "id" => ids.join(","),
			                                                         "maxResults" => 50 ].await;
			                                  (ids, channels)
		                                  })
		                                  .buffer_unordered(MAX_CON_REQUESTS)
		                                  .flat_map(|(ids, result)| { match result {
			                                  Err(err) => {
				                                  let err = Arc::new(err);
				                                  
				                                  ids.into_iter()
				                                     .map(move |id| (id, Err(err.clone())))
				                                     .into_box()
				                                     .into_stream()
			                                  },
			                                  Ok(mut subs) => {
				                                  skipped_errors.extend(subs.drain_filter(|c| c.is_err()).filter_map(Result::err));
		                                    
				                                  subs.into_iter()
				                                      .flatten()
				                                      .map(|sub| (sub.id, Ok(sub.content_details.related_playlists.uploads.clone())))
				                                      .into_box()
				                                      .into_stream()
			                                  }
		                                  }})
		                                  .collect::<Map<_>>()
		                                  .await;
		
		// Uploads -> Videos
		let uploads_videos = channel_uploads.values()
		                                    .flatten()
		                                    .cloned()
		                                    .into_stream()
		                                    .map(|uploads| async {
			                                    let videos = ytcall![self.api_key, "playlistItems", YouTubePlaylistItem,
			                                                         "part" => "snippet",
			                                                         "playlistId" => uploads,
			                                                         "maxResults" => 5; Single ].await;
			                                    
			                                    (uploads, videos)
		                                    })
		                                    .buffer_unordered(MAX_CON_REQUESTS)
		                                    .collect::<Map<_>>()
		                                    .await;
		
		config.into_iter()
		      .map(|(name, _)| {
			      let mut feed = Feed::new();
			      
			      for err in skipped_errors.iter() {
				      feed.add_err("Unexpected error while fetching.", &err.to_string());
			      }
			      
			      let channel = feed_channel.get(&name).flatten();
			      let channel = try_feed!(channel, feed, "Unable to find channel for {} feed.", name; return (name, feed));
			      
			      let subs = channel_subs.get(channel).flatten();
			      let subs = try_feed!(subs, feed, "Unable to get subscriptions for {} channel.", channel; return (name, feed));
			      
			      let empty_vec = vec![];
			      
			      let mut entries = subs.iter()
			                            .flat_map(|sub| {
				                            let sub = try_feed!(sub, feed, "Unable to fetch subscription for {} channel.", channel; return None);
				                            let sub = &sub.snippet.resource_id.channel_id;
				
				                            let uploads = channel_uploads.get(sub).flatten();
				                            let uploads = try_feed!(uploads, feed, "Unable to get uploads playlist for {} channel.", sub; return None);
				
				                            let videos = match uploads_videos.get(uploads).flatten() {
					                            Err(FetchError::HTTPError(err)) if err.status().unwrap_or_default() == 404 => Ok(&empty_vec), // Empty channels return 404 error
					                            videos => videos,
				                            };
				                            let videos = try_feed!(videos, feed, "Unable to get videos for {} channel, {} playlist.", sub, uploads; return None);
				
				                            Some(videos)
			                            })
			                            .flat_map(|videos| videos.iter())
			                            .map(|video| {
				                            let video = &video.as_ref().unwrap().snippet;
				                            Entry::new(&video.title, &video.resource_id.video_id)
				                                  .description(&video.description)
				                                  .extra(json!({ "displayName": &video.channel_title }))
				                                  .link(&format!("https://youtu.be/{}", video.resource_id.video_id))
				                                  .set_image_url(video.thumbnails.get("default").map(|tn| tn.url.clone()))
				                                  .set_timestamp(DateTime::parse_from_rfc3339(&video.published_at).ok())
			                            })
			                            .sorted()
			                            .collect::<Vec<_>>();
			      
			      feed.notifications.append(&mut entries);
			      
			      (name, feed)
		      })
		      .collect()
	}
}


#[derive(Debug, Error)]
pub enum FetchError {
	#[error(display = "API did not returned requested resource")] NotFound,
	#[error(display = "Client ID, Client Secret and Refresh Token are required to lookup own channel's subscriptions")] NoOauth,
	#[error(display = "{}", _0)] HTTPError(#[error(source)] reqwest::Error),
	#[error(display = "{}", _0)] JSONError(#[error(source)] serde_json::Error),
}


trait Flatten {
	type Output;
	fn flatten(self) -> Self::Output;
}

impl<'a, T> Flatten for Option<&'a Result<T, FetchError>> {
	type Output = Result<&'a T, &'a FetchError>;
	
	fn flatten(self) -> Self::Output {
		self.unwrap_or(&Err(FetchError::NotFound)).as_ref()
	}
}

impl<'a, T> Flatten for Option<&'a Result<T, Arc<FetchError>>> {
	type Output = Result<&'a T, Arc<FetchError>>;
	
	fn flatten(self) -> Self::Output {
		self.map(|r| r.as_ref().map_err(|err| err.clone()))
		    .unwrap_or(Err(Arc::new(FetchError::NotFound)))
	}
}

#[derive(Deserialize, Debug)]
struct YouTubeResponse {
	#[serde(rename="nextPageToken")]
	next_page_token: Option<String>,
	items: Vec<Json>,
}

#[derive(Deserialize, Debug)]
struct YouTubeSubscription {
	snippet: YouTubeSubscriptionSnippet,
}

#[derive(Deserialize, Debug)]
struct YouTubeSubscriptionSnippet {
	#[serde(rename="resourceId")]
	resource_id: YouTubeResourceChannelId,
}

#[derive(Deserialize, Debug)]
struct YouTubeResourceChannelId {
	#[serde(rename="channelId")]
	channel_id: String,
}

#[derive(Deserialize, Debug)]
struct YouTubeChannel {
	#[serde(rename="contentDetails")]
	content_details: YouTubeChannelDetails,
	id: String,
}

#[derive(Deserialize, Debug)]
struct YouTubeChannelDetails {
	#[serde(rename="relatedPlaylists")]
	related_playlists: YouTubeRelatedPlaylists,
}

#[derive(Deserialize, Debug)]
struct YouTubeRelatedPlaylists {
	uploads: String,
}

#[derive(Deserialize, Debug)]
struct YouTubePlaylistItem {
	snippet: YouTubePlaylistItemSnippet,
}

#[derive(Deserialize, Debug)]
struct YouTubePlaylistItemSnippet {
	#[serde(rename="publishedAt")]
	published_at: String,
	#[serde(rename="channelTitle")]
	channel_title: String,
	title: String,
	description: String,
	#[serde(rename="resourceId")]
	resource_id: YouTubeResourceVideoId,
	thumbnails: Map<YouTubeThumbnail>,
}

#[derive(Deserialize, Debug)]
struct YouTubeResourceVideoId {
	#[serde(rename="videoId")]
	video_id: String,
}

#[derive(Deserialize, Debug)]
struct YouTubeThumbnail {
	width: usize,
	height: usize,
	url: String,
}

#[derive(Deserialize, Debug)]
struct GoogleOauthResponse {
	access_token: String,
	expires_in: u64,
}
