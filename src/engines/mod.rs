use super::*;
use std::marker::PhantomData;

mod rss;

pub trait Engine : Sync {
    fn load_feed(&self, data: serde_json::Value) -> Feed;
}

pub fn find_engine(name: &str) -> &'static Engine {
    match name {
        "rss" => rss::ENGINE,
        _ => panic!("Cannot find {} engine.", name),
    }
}
