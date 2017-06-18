use super::*;

extern crate regex;
use self::regex::Regex;
use std::sync::mpsc;

#[derive(Deserialize)]
struct Request {
    command: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String
}

#[derive(Deserialize)]
struct FetchRequest {
    flat: bool,
    feeds: Vec<String>, // eg. "*", "funmaker-*", "mumble-twitch-wonziu"
}

#[derive(Serialize)]
struct FetchResponse<'a> {
    feeds: Map<&'a Feed>
}

#[derive(Serialize)]
struct FlatFetchResponse<'a> {
    status: Vec<&'a Entry>,
    notifications: Vec<&'a Entry>,
}

#[derive(Deserialize)]
struct ListRequest {
}

#[derive(Serialize)]
struct ListResponse {
    feeds: Map<ConfigFeedEntry>
}

#[derive(Deserialize)]
struct AddRequest {
    #[serde(rename="feedName")]
    feed_name: String,
    entry: ConfigFeedEntry,
}

#[derive(Serialize)]
struct AddResponse {
    #[serde(rename="feedName")]
    feed_name: String,
}

#[derive(Deserialize)]
struct RemoveRequest {
    #[serde(rename="feedName")]
    feed_name: String,
}

#[derive(Serialize)]
struct RemoveResponse {
    #[serde(rename="feedName")]
    feed_name: String,
}

#[derive(Deserialize)]
struct UpdateRequest;

#[derive(Serialize)]
pub struct UpdateResponse<'a> {
    pub status: Vec<&'a Entry>,
    pub notifications: Vec<&'a Entry>,
}

pub fn response_from_err(err: Box<Error>) -> Json {
    serde_json::to_value(ErrorResponse{
        error: format!("{}", err),
    }).unwrap()
}

pub fn handle_request(raw_request: Json, tx: &mpsc::Sender<Json>) -> Result<Json, Box<Error>> {
    let request: Request = try!(serde_json::from_value(raw_request.clone()));

    let mut response = match &*request.command {
        "fetch" => fetch(raw_request),
        "list" => list(raw_request),
        "add" => add(raw_request),
        "remove" => remove(raw_request),
        "update" => update(raw_request, tx),
        _ => HandleError::new(format!("Unknown command {}", request.command)),
    };

    if let Ok(Json::Object(ref mut response)) = response {
        response.insert("command".to_string(), Json::String(request.command));
    }

    response
}

fn fetch(request: Json) -> Result<Json, Box<Error>> {
    let request: FetchRequest = try!(serde_json::from_value(request));

    let feeds = get_feeds();
    let wrapped_filters = request.feeds.iter()
            .map(|filter| Regex::new(&format!("^{}", filter)));
    let mut filters = Vec::new();

    for filter in wrapped_filters {
        filters.push(try!(filter));
    }

    let matched: Map<&Feed> = feeds.iter()
            .filter(|&(name, _)| filters.iter()
                    .any(|filter| filter.is_match(name)))
            .map(|(name, feed)| (name.clone(), feed))
            .collect();

    if request.flat {
        Ok(serde_json::to_value(
            FlatFetchResponse{
                status: matched.iter().flat_map(|feed| feed.1.status.iter()).collect(),
                notifications: matched.iter().flat_map(|feed| feed.1.notifications.iter()).collect(),
            }
        ).unwrap())
    } else {
        Ok(serde_json::to_value(
            FetchResponse{
                feeds: matched
            }
        ).unwrap())
    }
}


fn list(request: Json) -> Result<Json, Box<Error>> {
    //let _request: ListRequest = try!(serde_json::from_value(request));
    let config = try!(load_config());

    Ok(serde_json::to_value(
        ListResponse{
            feeds: config.feeds
        }
    ).unwrap())
}

fn add(request: Json) -> Result<Json, Box<Error>> {
    let request: AddRequest = try!(serde_json::from_value(request));
    let mut config = try!(load_config());

    if let Some(_) = config.feeds.get(&request.feed_name) {
        HandleError::new(format!("Feed {} already exsists.", request.feed_name))
    } else {
        let name = request.feed_name;
        match maybe_fetch_feed(&name, &request.entry) {
            Ok(feed) => get_feeds().insert(name.clone(), feed),
            Err(err) => return HandleError::new(format!("Unable to fetch feed: {}", err)),
        };
        config.feeds.insert(name.clone(), request.entry);
        save_config(&config).map(|()| serde_json::to_value(
            AddResponse{
                feed_name: name,
            }
        ).unwrap())
    }
}

fn remove(request: Json) -> Result<Json, Box<Error>> {
    let request: RemoveRequest = try!(serde_json::from_value(request));
    let mut config = try!(load_config());

    if let Some(_) = config.feeds.remove(&request.feed_name) {
        let name = request.feed_name;
        get_feeds().remove(&name);
        save_config(&config).map(|()| serde_json::to_value(
            RemoveResponse{
                feed_name: name,
            }
        ).unwrap())
    } else {
        HandleError::new(format!("Feed {} doesn't exsist.", request.feed_name))
    }
}

fn update(request: Json, tx: &mpsc::Sender<Json>) -> Result<Json, Box<Error>> {
    //let _request: UpdateRequest = try!(serde_json::from_value(request));

    add_updater(tx);

    Ok(serde_json::to_value(UpdateResponse {
        status: vec![],
        notifications: vec![],
    }).unwrap())
}
