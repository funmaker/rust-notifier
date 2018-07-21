use super::*;

use std::collections::BTreeMap;
use std::time::{Duration};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Serialize)]
pub struct Entry {
    pub title: String,
    pub guid: String,
    #[serde(rename="feedName")]
    pub feed_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub color: Option<String>,
    #[serde(rename="imageURL")]
    pub image_url: Option<String>,
    pub timestamp: Option<u64>,
    pub error: bool,
    pub extra: Option<Json>,
}

impl Entry {
    pub fn new(title: &str, guid: &str) -> Entry {
        Entry{
            title: title.to_string(),
            guid: guid.to_string(),
            feed_name: None,
            description: None,
            link: None,
            color: None,
            image_url: None,
            timestamp: None,
            error: false,
            extra: None,
        }
    }

    pub fn description(mut self, description: &str) -> Entry {
        self.description = Some(description.to_string());
        self
    }

    pub fn link(mut self, link: &str) -> Entry {
        self.link = Some(link.to_string());
        self
    }

    pub fn color(mut self, color: &str) -> Entry {
        self.color = Some(color.to_string());
        self
    }

    pub fn image_url(mut self, image_url: &str) -> Entry {
        self.image_url = Some(image_url.to_string());
        self
    }

    pub fn timestamp(mut self, timestamp: u64) -> Entry {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn error(mut self, error: bool) -> Entry {
        self.error = error;
        self
    }

    pub fn extra(mut self, extra: Json) -> Entry {
        self.extra = Some(extra);
        self
    }

    pub fn feed_name(mut self, feed_name: &str) -> Entry {
        self.feed_name = Some(feed_name.to_string());
        self
    }

    pub fn set_description(mut self, description: Option<String>) -> Entry {
        self.description = description;
        self
    }

    pub fn set_link(mut self, link: Option<String>) -> Entry {
        self.link = link;
        self
    }

    pub fn set_color(mut self, color: Option<String>) -> Entry {
        self.color = color;
        self
    }

    pub fn set_image_url(mut self, image_url: Option<String>) -> Entry {
        self.image_url = image_url;
        self
    }

    pub fn set_timestamp(mut self, timestamp: Option<u64>) -> Entry {
        self.timestamp = timestamp;
        self
    }

    pub fn set_error(mut self, error: bool) -> Entry {
        self.error = error;
        self
    }

    pub fn set_extra(mut self, extra: Option<Json>) -> Entry {
        self.extra = extra;
        self
    }

    pub fn set_feed_name(mut self, feed_name: Option<String>) -> Entry {
        self.feed_name = feed_name;
        self
    }
}

#[derive(Serialize)]
pub struct Feed {
    pub status: Vec<Entry>,
    pub notifications: Vec<Entry>,
}

impl Feed {
    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Entry> {
        self.status.iter_mut().chain(self.notifications.iter_mut())
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Entry> {
        self.status.iter().chain(self.notifications.iter())
    }

    pub fn new() -> Feed {
        Feed {
            status: Vec::new(),
            notifications: Vec::new()
        }
    }

    pub fn from_err(err: &str, desc: &str) -> Feed {
        Feed {
            status: vec![
                Entry::new(err, &hash(&(timestamp(), err, desc)))
                    .description(desc)
                    .color("#FF0000")
                    .timestamp(timestamp())
            ],
            notifications: Vec::new(),
        }
    }
}

pub type Map<T> = BTreeMap<String, T>;
pub struct Feeds {
    pub created: u64,
    pub feeds: Map<Feed>,
}
impl Feeds {
    pub fn new() -> Self {Feeds{
        created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        feeds: Map::new(),
    }}
}
impl std::ops::Deref for Feeds {
    type Target = Map<Feed>;

    fn deref(&self) -> &Map<Feed> {
        &self.feeds
    }
}
impl std::ops::DerefMut for Feeds {
    fn deref_mut(&mut self) -> &mut Map<Feed> {
        &mut self.feeds
    }
}

pub fn fetch_feeds() {
    let mut feeds = BTreeMap::new();

    let config = load_config().unwrap();

    for (name, data) in &config.feeds {
        let feed = fetch_feed(name, data);
        feeds.insert(name.clone(), feed);
    }

    let feeds = Feeds{
        created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        feeds,
    };

    run_update(&feeds);
    *get_feeds() = feeds;
}

pub fn start_fetch_thread(interval: Duration) -> thread::JoinHandle<()> {
    use std::time::Instant;
    use std::io::{self, Write};
    thread::spawn(move || {
        loop{
            let now = Instant::now();
            print!("Fetching feeds... ");
            let _ = io::stdout().flush();

            fetch_feeds();

            let dur = now.elapsed();
            println!("Done! ({}s)", dur.as_secs());

            thread::sleep(interval);
        }
    })
}
