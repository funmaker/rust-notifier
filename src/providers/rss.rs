extern crate rss;

use super::super::*;
use self::rss::Channel;
use std::io::BufReader;
pub static PROVIDER: &'static Provider = &RSSEngine;

struct RSSEngine;

impl Provider for RSSEngine {
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>> {
        let mut feed = Feed::new();
        let url = try!(data.as_str().ok_or("engineData should be an String."));
        
        let chan = try!(Channel::read_from(BufReader::new( &try!(http_get(url))[..] )));
        
        for item in chan.items {
            let entry = Entry::new(item.title.as_ref().unwrap_or(&"<No Title>".to_string()))
                    .set_description(item.description.or(item.content.clone()))
                    .set_link(item.link)
                    .set_guid(item.guid.map(|g| g.clone().value))
                    .set_timestamp(item.pub_date
                            .and_then(|time| time::strptime(&time, "%a, %d %b %Y %T %z")
                            .ok()
                            .map(|tm| to_timestamp(tm))));
            
            feed.notifications.push(entry);
        }
        
        Ok(feed)
    }
}
