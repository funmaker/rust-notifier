use super::*;

mod rss;

lazy_static! {
    static ref ENABLED_PROVIDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub trait Provider : Sync {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>{
        None
    }
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>>;
}

fn find_provider(name: &str) -> &'static Provider {
    match name {
        "rss" => rss::PROVIDER,
        _ => panic!("Cannot find {} provider.", name),
    }
}

#[derive(Deserialize)]
struct BaseSettings {
    enabled: bool,
}

pub fn start_providers(config: &Map<Json>) -> Map<Option<thread::JoinHandle<()>>> {
    let mut map = Map::new();
    for (name, config) in config {
        let settings: BaseSettings = serde_json::from_value(config.clone()).unwrap();
        if settings.enabled == false {
            continue
        }
        map.insert(name.clone(), find_provider(&name).start(config));
        ENABLED_PROVIDERS.lock().unwrap().push(name.clone());
    }
    map
}

pub fn fetch_feed(feed_name: &str, feed_data: &ConfigFeedEntry) -> Feed {
    maybe_fetch_feed(feed_name, feed_data)
            .unwrap_or_else(|err| Feed{
                notifications: vec![],
                status: vec![
                    Entry::new(&format!("Unable to fetch, {}", err))
                            .timestamp(timestamp())
                            .color("#FF0000")
                ]
            })
}

pub fn maybe_fetch_feed(feed_name: &str, feed_data: &ConfigFeedEntry) -> Result<Feed, Box<Error>> {
    {
        if ENABLED_PROVIDERS.lock().unwrap().iter().all(|p| p != &feed_data.provider) {
            return HandleError::new("Provider is disabled".to_string());
        }
    }
    
    let mut feed = find_provider(&feed_data.provider).load_feed(&feed_data.provider_data);
    
    if let Ok(ref mut feed) = feed {
        for entry in feed.iter_mut() {
            if entry.color.is_none() {
                entry.color = feed_data.color.clone();
            }
            entry.feed_name = Some(feed_name.to_string());
        }
    }
    
    feed
}