extern crate regex;

use super::super::*;
use self::regex::RegexBuilder;
pub static PROVIDER: &'static Provider = &ChanProvider;

struct ChanProvider;

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
    tim: Option<u64>,
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

impl Provider for ChanProvider {
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>> {
        let mut feed = Feed::new();
        let data: Data = try!(serde_json::from_value(data.clone()));
        
        let wrapped_filters = data.filters.iter()
                .map(|filter| RegexBuilder::new(filter)
                        .case_insensitive(true)
                        .compile());
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
                    let mut entry = Entry::new(&op.sub.unwrap_or(op.semantic_url.replace("-", " ")), &hash(&(op.no, board)))
                            .set_description(op.com)
                            .link(&format!("https://boards.4chan.org/{}/thread/{}", board, op.no))
                            .timestamp(op.time as u64)
                            .extra(extra);
                    if let Some(tim) = op.tim {
                        entry = entry.image_url(&format!("https://i.4cdn.org/{}/{}s.jpg", board, tim));
                    }
                    feed.status.push(entry);
                }
            }
        }
        
        Ok(feed)
    }
}
