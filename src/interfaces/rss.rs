extern crate hyper;
extern crate rss;
use super::super::*;

use self::hyper::{Body, Request, Response, Server};
use self::hyper::rt::Future;
use self::hyper::service::service_fn_ok;
use self::rss::{ChannelBuilder, ItemBuilder, Item, CategoryBuilder, GuidBuilder};

pub static INTERFACE: &'static Interface = &RSSInterface;

struct RSSInterface;

#[derive(Deserialize)]
struct Settings {
    port: u16,
}

struct Cache {
    timestamp: u64,
    rendered: String,
}
lazy_static! {
    static ref CACHE: Mutex<Option<Cache>> = Mutex::new(None);
}

fn handle_request(_req: Request<Body>) -> Response<Body> {
    let mut cache = CACHE.lock().unwrap();
    {
        let feed = get_feeds();
        if cache.is_none() || cache.as_ref().unwrap().timestamp != feed.created {
            let channel = ChannelBuilder::default()
                .title("Rust Notifier")
                .items(feed.values()
                    .flat_map(|feed| &feed.status)
                    .chain(feed.values().flat_map(|feed| &feed.notifications))
                    .map(|entry|
                        ItemBuilder::default()
                            .title(entry.title.clone())
                            .guid(
                                GuidBuilder::default()
                                    .value(entry.guid.clone())
                                    .build()
                                    .unwrap())
                            .categories(vec![
                                CategoryBuilder::default()
                                    .name(entry.feed_name.as_ref().unwrap().clone())
                                    .build()
                                    .unwrap()])
                            .link(entry.link.clone())
                            .description(entry.description.clone())
                            .build()
                            .unwrap())
                    .take(50)
                    .collect::<Vec<Item>>())
                .build()
                .unwrap();

            *cache = Some(Cache{
                timestamp: feed.created,
                rendered: channel.to_string(),
            });
        }
    }

    Response::builder()
        .header("Content-Type", "application/xml; charset=UTF-8")
        .body(Body::from(cache.as_ref().unwrap().rendered.clone()))
        .unwrap()
}

impl Interface for RSSInterface {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>> {
        use std::thread;


        let settings: Settings = serde_json::from_value(config.clone()).unwrap();
        let addr = ([127, 0, 0, 1], settings.port).into();

        let server = Server::bind(&addr)
            .serve(|| service_fn_ok(handle_request))
            .map_err(|e| println!("server error: {}", e));

        Some(thread::spawn(move || {
            hyper::rt::run(server);
        }))
    }
}
