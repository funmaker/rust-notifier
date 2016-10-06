extern crate rss;

use super::super::*;
use self::rss::Channel;
use std::io::BufReader;
pub static PROVIDER: &'static Provider = &RSSProvider;

struct RSSProvider;

impl Provider for RSSProvider {
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>> {
        let mut feed = Feed::new();
        let url: String = try!(serde_json::from_value(data.clone()));
        
        let chan = try!(Channel::read_from(BufReader::new( &try!(http_get(&url))[..] )));
        
        for item in chan.items {
            let title = item.title.clone().unwrap_or("<No Title>".to_string());
            let timestamp = item.pub_date
                    .and_then(|time| time::strptime(&time, "%a, %d %b %Y %T %z")
                            .ok()
                            .map(|tm| to_timestamp(tm)));
            let guid = item.guid.clone()
                    .map(|g| g.value)
                    .unwrap_or_else(|| hash(&(&title, timestamp)));
            let entry = Entry::new(&title, &guid)
                    .set_description(item.description.or(item.content.clone()))
                    .set_link(item.link)
                    .set_timestamp(timestamp);
            
            feed.notifications.push(entry);
        }
        
        Ok(feed)
    }
}
