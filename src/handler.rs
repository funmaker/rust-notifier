use super::*;

extern crate regex;
use self::regex::Regex;

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
    command: String,
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
    command: String,
}

#[derive(Serialize)]
struct ListResponse {
    feeds: Map<ConfigFeedEntry>
}

#[derive(Deserialize)]
struct AddRequest {
    command: String,
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
    command: String,
    feeds: Vec<String>,
}

#[derive(Serialize)]
struct RemoveResponse {
    #[serde(rename="feedName")]
    feed_name: String,
}

pub fn handle_request(request: serde_json::Value) -> serde_json::Value {
    maybe_handle_request(request)
            .unwrap_or_else(|err| serde_json::to_value(
                    ErrorResponse{
                        error: err.description().to_string()
                    }))
}


fn maybe_handle_request(raw_request: serde_json::Value) -> Result<serde_json::Value, Box<Error>> {
    let request: Request = try!(serde_json::from_value(raw_request.clone()));
    
    match &*request.command {
        "fetch" => fetch(raw_request),
        "list" => list(raw_request),
        "add" => add(raw_request),
        "remove" => remove(raw_request),
        _ => HandleError::new(format!("Unknown command {}", request.command)),
    }
}

fn fetch(request: serde_json::Value) -> Result<serde_json::Value, Box<Error>> {
    let request: FetchRequest = try!(serde_json::from_value(request));
    
    let feeds = get_feeds();
    let wrapped_filters = request.feeds.iter()
            .map(|filter| Regex::new(&format!("^{}", filter)));
    let mut filters = Vec::new();
    
    for filter in wrapped_filters {
        filters.push(try!(filter));
    }
    
    let matched: Map<&Feed> = feeds.iter()
            .filter(|&(name, feed)| filters.iter()
                    .any(|filter| filter.is_match(name)))
            .map(|(name, feed)| (name.clone(), feed))
            .collect();
    
    if request.flat {
        Ok(serde_json::to_value(
            FlatFetchResponse{
                status: matched.iter().flat_map(|feed| feed.1.status.iter()).collect(),
                notifications: matched.iter().flat_map(|feed| feed.1.notifications.iter()).collect(),
            }
        ))
    } else {
        Ok(serde_json::to_value(
            FetchResponse{
                feeds: matched
            }
        ))
    }
}


fn list(request: serde_json::Value) -> Result<serde_json::Value, Box<Error>> {
    let request: ListRequest = try!(serde_json::from_value(request));
    let config = load_config();
    
    Ok(serde_json::to_value(
        ListResponse{
            feeds: config.feeds
        }
    ))
}

fn add(request: serde_json::Value) -> Result<serde_json::Value, Box<Error>> {
    let request: AddRequest = try!(serde_json::from_value(request));
    let mut config = load_config();
    
    if let Some(_) = config.feeds.get(&request.feed_name) {
        HandleError::new(format!("Feed {} already exsists.", request.feed_name))
    } else {
        config.feeds.insert(request.feed_name.clone(), request.entry);
        let name = request.feed_name;
        save_config(&config).map(|()| serde_json::to_value(
            AddResponse{
                feed_name: name,
            }
        ))
    }
}

fn remove(request: serde_json::Value) -> Result<serde_json::Value, Box<Error>> {
    let request: RemoveRequest = try!(serde_json::from_value(request));
    unimplemented!()
}
