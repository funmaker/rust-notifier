extern crate regex;

use super::super::*;
use std::io::BufReader;
use self::regex::Regex;
pub static PROVIDER: &'static Provider = &ChanEngine;

struct ChanEngine;

#[derive(Deserialize)]
struct Data {
    boards: Vec<String>,
    filters: Vec<String>,
}

type Catalog = Vec<Page>;

#[derive(Deserialize)]
struct Page {
    page: i32,
    threads: Vec<OP>,
}

#[derive(Deserialize, Clone)]
struct OP {
    no: i32,
    time: u64,
    sub: Option<String>,
    com: Option<String>,
    tim: u64,
    ext: String,
    replies: i32,
    images: i32,
    semantic_url: String,
}

#[derive(Serialize)]
struct OPExtra {
    replies: i32,
    images: i32,
    page: i32,
    board: String,
}

impl Provider for ChanEngine {
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>> {
        let mut feed = Feed::new();
        let data: Data = try!(serde_json::from_value(data.clone()));
        
        let wrapped_filters = data.filters.iter()
                .map(|filter| Regex::new(filter));
        let mut filters = Vec::new();
        
        for filter in wrapped_filters {
            filters.push(try!(filter));
        }
        
        for board in &data.boards {
            let url = format!("http://a.4cdn.org/{}/catalog.json", board);
            let catalog: Catalog = try!(serde_json::from_slice(&try!(http_get(&url))));
            for page in catalog {
                for op in page.threads {
                    if !filters.iter().any(|f| f.is_match(op.sub.as_ref().unwrap_or(&"".to_string())) || f.is_match(op.com.as_ref().unwrap_or(&"".to_string()))) {
                        continue;
                    }
                    let extra = serde_json::to_value(OPExtra{
                        replies: op.replies,
                        images: op.images,
                        page: page.page,
                        board: board.clone(),
                    });
                    feed.status.push(Entry::new(&op.sub.unwrap_or(op.semantic_url), &hash(&(op.no, board)))
                            .set_description(op.com)
                            .link(&format!("https://boards.4chan.org/{}/thread/{}", board, op.no))
                            .image_url(&format!("https://i.4cdn.org/{}/{}.{}", board, op.tim, op.ext))
                            .timestamp(op.time as u64)
                            .extra(extra));
                }
            }
        }
        
        Ok(feed)
    }
}