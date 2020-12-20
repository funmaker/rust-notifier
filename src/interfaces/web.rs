use std::error::Error;
use std::convert::Infallible;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply, reply, reject};
use warp::http::StatusCode;
use warp::reject::Reject;
use regex::RegexBuilder;
use futures::future;

use crate::utils::{Json, Map};
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
}

pub async fn serve(config: Json, state: State) -> Result<(), Box<dyn Error>> {
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
	     .and_then(move |query: FetchQuery| future::ready::<Result<reply::Json, Rejection>>(try {
		     let filter = query.filter.map(|filter| RegexBuilder::new(&filter)
		                                                         .size_limit(1024 * 32)
		                                                         .dfa_size_limit(1024 * 32)
		                                                         .nest_limit(10)
		                                                         .build())
		                       .transpose()
		                       .map_err(RegexpReject)
		                       .map_err(reject::custom)?;
		
		     let feeds = state.feeds.load();
		     let feeds = feeds.iter()
		                      .filter(|(name, _)| filter.as_ref().map_or(true, |reg| reg.is_match(name)))
		                      .map(|(name, feed)| (name.clone(), feed.clone()));
		
		     let json = if query.flat.unwrap_or(false) {
			     reply::json(&feeds.fold(Feed::new(), |acc, (_, feed)| acc.append(feed)))
		     } else {
			     reply::json(&feeds.collect::<Map<Feed>>())
		     };
		     
		     json
	     }))
	     .with(warp::cors().allow_any_origin()).with(warp::log("cors test"))
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
	} else if let Some(_) = err.find::<warp::reject::InvalidQuery>() {
		code = StatusCode::BAD_REQUEST;
	} else if let Some(e) = err.find::<RegexpReject>() {
		code = StatusCode::BAD_REQUEST;
		message = Some(e.0.to_string());
	} else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
		code = StatusCode::METHOD_NOT_ALLOWED;
	} else {
		eprintln!("unhandled rejection: {:?}", err);
		code = StatusCode::INTERNAL_SERVER_ERROR;
	}
	
	let json = warp::reply::json(&ErrorMessage {
		code: code.as_u16(),
		message: message.unwrap_or(code.to_string()),
	});
	
	Ok(warp::reply::with_status(json, code))
}
