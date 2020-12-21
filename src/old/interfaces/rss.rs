extern crate hyper;
extern crate rss;
extern crate url;
use super::super::*;

use self::hyper::{Body, Request, Response, Server};
use self::hyper::rt::Future;
use self::hyper::service::service_fn_ok;
use self::rss::{ChannelBuilder, ItemBuilder, Item, CategoryBuilder, GuidBuilder};
use self::rss::extension::{Extension, ExtensionBuilder};
use self::url::Url;
use std::collections::HashMap;

pub static INTERFACE: &'static Interface = &RSSInterface;

struct RSSInterface;

#[derive(Deserialize)]
struct Settings {
    port: u16,
}

fn handle_request(req: Request<Body>) -> Response<Body> {
    let feed = get_feeds();

    let query: HashMap<String, String> = Url::parse(&format!("http://localhost{}", req.uri())).ok().map_or(HashMap::new(), |url| url.query_pairs().into_owned().collect());

    let body = ChannelBuilder::default()
        .title("Rust Notifier")
        .items(feed.iter()
            .filter_map(|(name, feed)| query.get("filter").map_or(Some(feed), |filter| if name.starts_with(filter) {Some(feed)} else {None}))
            .flat_map(|feed| feed.status.iter())
            .chain(feed.iter()
                .filter_map(|(name, feed)| query.get("filter").map_or(Some(feed), |filter| if name.starts_with(filter) {Some(feed)} else {None}))
                .flat_map(|feed| feed.notifications.iter())
                .sorted_by(|a, b| b.timestamp.cmp(&a.timestamp)))
            .take(50)
            .map(|entry|
                ItemBuilder::default()
                    .title(entry.title.clone())
                    .guid(GuidBuilder::default()
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
                    .pub_date(entry.timestamp.map(|ts| format!("{}", from_timestamp(ts).rfc822())))
                    .extensions(
                        hashmap!{
                        "x-notifier".to_string() => hashmap!{
                                "x-notifier".to_string() => vec![
                                    ExtensionBuilder::default()
                                        .name("x-notifier-color")
                                        .value(entry.color.clone())
                                        .build()
                                        .unwrap(),
                                    ExtensionBuilder::default()
                                        .name("x-notifier-extra")
                                        .children(entry.extra.clone().and_then(|extra|
                                            extra.as_object().map(|extra|
                                                extra.iter().map(|(name, entry)|
                                                    match entry {
                                                        Json::String(s) => (name, s.to_string()),
                                                        any => (name, serde_json::to_string(any).unwrap()),
                                                    })
                                                .map(|(name, value)| (name.to_string(), vec![
                                                    ExtensionBuilder::default()
                                                        .name(name.to_string())
                                                        .value(value)
                                                        .build()
                                                        .unwrap()]))
                                                .collect::<HashMap<String, Vec<Extension>>>()))
                                            .unwrap_or(HashMap::new()))
                                        .build()
                                        .unwrap(),
                                    ]}})
                    .build()
                    .unwrap())
            .collect::<Vec<Item>>())
        .build()
        .unwrap()
        .to_string();

    Response::builder()
        .header("Content-Type", "application/xml; charset=UTF-8")
        .body(Body::from(body))
        .unwrap()
}

impl Interface for RSSInterface {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>> {
        use std::thread;


        let settings: Settings = serde_json::from_value(config.clone()).unwrap();
        let addr = ([0, 0, 0, 0], settings.port).into();

        let server = Server::bind(&addr)
            .serve(|| service_fn_ok(handle_request))
            .map_err(|e| println!("server error: {}", e));

        Some(thread::spawn(move || {
            hyper::rt::run(server);
        }))
    }
}
