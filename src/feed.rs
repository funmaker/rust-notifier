use super::*;

use std::collections::BTreeMap;

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
    
    pub fn extra(mut self, extra: Json) -> Entry {
        self.extra = Some(extra);
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
    
    pub fn set_extra(mut self, extra: Option<Json>) -> Entry {
        self.extra = extra;
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
pub type Feeds = Map<Feed>;

pub fn fetch_feeds() {
    let mut feeds = BTreeMap::new();
    
    let config = load_config().unwrap();
    
    for (name, data) in &config.feeds {
        println!("Fetching: {}", name);
        let feed = fetch_feed(name, data);
        feeds.insert(name.clone(), feed);
    }
    
    *get_feeds() = feeds;
}

pub fn start_fetch_thread(interval: Duration) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        thread::sleep(interval);
        fetch_feeds();
    })
}
