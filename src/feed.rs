use super::*;

use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct Entry {
    pub title: String,
    pub feed_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub guid: Option<String>,
    pub color: Option<String>,
    pub image_url: Option<String>,
    pub timestamp: Option<u64>,
}

impl Entry {
    pub fn new(title: &str) -> Entry {
        Entry{
            title: title.to_string(),
            feed_name: None,
            description: None,
            link: None,
            guid: None,
            color: None,
            image_url: None,
            timestamp: None,
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
    
    pub fn guid(mut self, guid: &str) -> Entry {
        self.link = Some(guid.to_string());
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
    
    pub fn set_description(mut self, description: Option<String>) -> Entry {
        self.description = description.clone();
        self
    }
    
    pub fn set_link(mut self, link: Option<String>) -> Entry {
        self.link = link.clone();
        self
    }
    
    pub fn set_guid(mut self, guid: Option<String>) -> Entry {
        self.link = guid.clone();
        self
    }
    
    pub fn set_color(mut self, color: Option<String>) -> Entry {
        self.color = color.clone();
        self
    }
    
    pub fn set_image_url(mut self, image_url: Option<String>) -> Entry {
        self.image_url = image_url.clone();
        self
    }
    
    pub fn set_timestamp(mut self, timestamp: Option<u64>) -> Entry {
        self.timestamp = timestamp.clone();
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
                Entry::new(err)
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
