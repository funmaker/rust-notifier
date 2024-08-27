use std::collections::BTreeMap;
use std::convert::Infallible;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply, reply, reject};
use warp::http::StatusCode;
use warp::reject::Reject;
use regex::RegexBuilder;
use futures::future;
use rss::{ChannelBuilder, ItemBuilder, CategoryBuilder, GuidBuilder};
use rss::extension::{Extension, ExtensionBuilder};
use anyhow::Result;

use crate::utils::{Json, Map, IteratorEx};
use crate::state::State;
use crate::feeds::Feed;

#[derive(Deserialize)]
struct WebConfig {
	rest: bool,
	rss: bool,
	websocket: bool,
	port: u16,
}

#[derive(Deserialize)]
struct FetchQuery {
	filter: Option<String>,
	flat: Option<bool>,
	format: Option<Format>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Format {
	JSON,
	RSS,
}

pub async fn serve(config: Json, state: State) -> Result<()> {
	let config: WebConfig = serde_json::from_value(config)?;
	
	let fetch = feeds_get(state.clone());
	
	let routes = fetch.recover(handle_rejection);
	
	println!("Serving web on port {}", config.port);
	warp::serve(routes)
	     .run(([0, 0, 0, 0], config.port))
	     .await;
	
	Ok(())
}


// GET /feeds?filter=my-feed&flat=true
fn feeds_get(state: State) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	warp::get()
	     .and(warp::path("feeds"))
	     .and(warp::query())
	     .and_then(move |query: FetchQuery| future::ready::<Result<Box<dyn Reply>, Rejection>>(try {
		     let filter = query.filter.map(|filter| RegexBuilder::new(&filter)
		                                                         .size_limit(1024 * 32)
		                                                         .dfa_size_limit(1024 * 32)
		                                                         .nest_limit(10)
		                                                         .case_insensitive(true)
		                                                         .build())
		                       .transpose()
		                       .map_err(RegexpReject)
		                       .map_err(reject::custom)?;
		     
		     let feeds = state.feeds.load();
		     let feeds = feeds.iter()
		                      .filter(|(name, _)| filter.as_ref().map_or(true, |reg| reg.is_match(name)))
		                      .map(|(name, feed)| (name.clone(), feed));
		     
		     
		     if let Some(Format::RSS) = query.format {
			     Box::new(generate_rss(feeds.map(|(_, feed)| feed).kmerge_feeds())) as Box<dyn Reply>
		     } else if query.flat.unwrap_or(false) {
			     Box::new(reply::json(&feeds.map(|(_, feed)| feed).kmerge_feeds())) as Box<dyn Reply>
		     } else {
			     Box::new(reply::json(&feeds.collect::<Map<&Feed>>())) as Box<dyn Reply>
		     }
	     }))
	     .with(warp::cors().allow_any_origin()).with(warp::log("cors test"))
}

fn map<T>(key: &str, value: T) -> BTreeMap<String, T> {
	let mut map = BTreeMap::new();
	map.insert(key.to_string(), value);
	map
}

fn generate_extension(value: Json, name: String) -> Extension {
	let mut builder = ExtensionBuilder::default();
	builder.name(name.to_string());
	
	match value {
		Json::Null => &mut builder,
		Json::Bool(value) => builder.value(value.to_string()),
		Json::Number(value) => builder.value(value.to_string()),
		Json::String(value) => builder.value(value),
		Json::Array(value) => builder.children(value.into_iter()
		                                            .enumerate()
		                                            .map(|(n, value)| (n.to_string(), vec![generate_extension(value, n.to_string())]))
		                                            .collect::<BTreeMap<_, _>>()),
		Json::Object(value) => builder.children(value.into_iter()
		                                             .map(|(key, value)| (key.clone(), vec![generate_extension(value, key)]))
		                                             .collect::<BTreeMap<_, _>>()),
	}.build()
}

fn generate_rss(feed: Feed) -> impl Reply {
	let body = ChannelBuilder::default()
		.title("Rust Notifier")
		.items(feed.status
		           .iter()
		           .chain(feed.notifications.iter())
		           .take(50)
		           .map(|entry|
			           ItemBuilder::default()
				                   .title(entry.title.clone())
				                   .guid(GuidBuilder::default()
					                                 .value(entry.guid.clone())
					                                 .build())
				                   .categories([CategoryBuilder::default()
				                                                .name(entry.feed_name.as_ref().unwrap().clone())
				                                                .build()])
				                   .link(entry.link.clone())
				                   .description(entry.description.clone())
				                   .pub_date(entry.timestamp.map(|ts| ts.to_rfc2822()))
				                   .extensions(map("x-notifier", map("x-notifier",
					                   vec![
						                   entry.color.clone().map(|color|
							                   ExtensionBuilder::default()
							                                    .name("x-notifier-color")
							                                    .value(color)
							                                    .build()
						                   ),
						                   entry.extra.clone().map(|extra| generate_extension(extra, "x-notifier-extra".to_string())),
					                   ]
					                   .into_iter()
					                   .flatten()
					                   .collect()
				                   )))
				                   .build())
		           .collect::<Vec<_>>())
		.build()
		.to_string();
	
	reply::with_header(body, "Content-Type", "application/xml; charset=UTF-8")
}

#[derive(Debug)]
struct RegexpReject(regex::Error);
impl Reject for RegexpReject {}

#[derive(Serialize)]
struct ErrorMessage {
	code: u16,
	message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
	let code;
	let mut message = None;
	
	if err.is_not_found() {
		code = StatusCode::NOT_FOUND;
	} else if let Some(_) = err.find::<reject::InvalidQuery>() {
		code = StatusCode::BAD_REQUEST;
	} else if let Some(e) = err.find::<RegexpReject>() {
		code = StatusCode::BAD_REQUEST;
		message = Some(e.0.to_string());
	} else if let Some(_) = err.find::<reject::MethodNotAllowed>() {
		code = StatusCode::METHOD_NOT_ALLOWED;
	} else {
		eprintln!("unhandled rejection: {:?}", err);
		code = StatusCode::INTERNAL_SERVER_ERROR;
	}
	
	let json = reply::json(&ErrorMessage {
		code: code.as_u16(),
		message: message.unwrap_or(code.to_string()),
	});
	
	Ok(reply::with_status(json, code))
}
