use async_trait::async_trait;
use futures::TryFutureExt;
use serde::Deserialize;
use serde_json::json;
use chrono::DateTime;
use anyhow::Result;

use super::Provider;
use crate::utils::{Json, Map, hash};
use crate::config::ConfigFeedEntry;
use crate::feeds::{Feed, Entry};

pub struct VinesauceProvider;

impl VinesauceProvider {
	pub fn new(_config: Json) -> Result<Self> {
		Ok(VinesauceProvider)
	}
}

type TeamData = Map<Member>;

#[derive(Deserialize)]
struct ProviderData {
	channels: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Member {
	stream: Data<Stream>,
	channel: Data<Channel>,
}

#[derive(Deserialize)]
struct Data<T> {
	data: Vec<T>
}

#[derive(Deserialize)]
struct Stream {
	user_name: String,
	game_name: String,
	title: String,
	started_at: String,
	thumbnail_url: String,
}

#[derive(Deserialize)]
struct Channel {
	login: String,
}

#[async_trait(?Send)]
impl Provider for VinesauceProvider {
	async fn fetch(&mut self, config: Map<&ConfigFeedEntry>, _client: reqwest::Client) -> Map<Feed> {
		let team_data = reqwest::get("https://vinesauce.com/twitch/team-data-helix.json")
		                        .map_err(anyhow::Error::new)
		                        .and_then(|res| res.bytes().map_err(Into::into))
		                        .await
		                        .map(|bytes| serde_json::from_slice::<TeamData>(&*bytes).map_err(anyhow::Error::new));
		
		config.into_iter()
		      .map(|(name, config)| {
			      let channels = match serde_json::from_value::<Option<ProviderData>>(config.provider_data.clone()) {
				      Ok(provider_data) => provider_data.map(|provider_data| provider_data.channels).flatten(),
				      Err(err) => return (name, Feed::from_err("Unable to parse providerData", &err.into())),
			      };
			      
			      match &team_data {
				      Ok(Ok(team_data)) => {
					      let mut feed = Feed::new();
					      
					      for member in team_data.values() {
						      if let Some(stream) = member.stream.data.get(0) {
							      if let Some(channel) = member.channel.data.get(0) {
								      if !channels.as_ref().map_or(true, |channels| channels.contains(&channel.login)) { continue; }
								      
								      feed.status.push(
									      Entry::new(&stream.title, &hash(&(&stream.started_at, "vinesauce")))
									            .set_timestamp(DateTime::parse_from_rfc3339(&stream.started_at).ok())
									            .link("http://vinesauce.com/")
									            .description(&stream.game_name)
									            .image_url(&stream.thumbnail_url.replace("{width}", "250").replace("{height}", "140"))
									            .extra(json!({ "displayName": stream.user_name }))
								      )
							      }
						      }
					      }
					      
					      (name, feed)
				      }
				      Ok(Err(err)) => (name, Feed::from_err("Failed to parse team data.", err)),
				      Err(err) => (name, Feed::from_err("Failed to fetch team data.", err)),
			      }
		      })
		      .collect()
	}
}
