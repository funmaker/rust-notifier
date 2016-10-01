#![feature(custom_derive)]
#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![plugin(serde_macros)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub extern crate serde_json;
pub extern crate time;
#[macro_use] extern crate lazy_static;

pub use std::sync::{Arc, Mutex, MutexGuard};
pub use std::error::Error;
pub use std::time::{Duration, Instant};
pub use std::thread;
mod utils;
mod feed;
mod engines;
mod websocket;
mod handler;
pub use utils::*;
pub use feed::*;
pub use engines::*;
pub use websocket::*;
pub use handler::*;

lazy_static! {
    static ref FEEDS: Mutex<Feeds> = Mutex::new(Feeds::new());
}

pub fn get_feeds() -> MutexGuard<'static, Feeds> {
    FEEDS.lock().unwrap()
}

fn main() {
    *get_feeds() = load_feeds();
    let loading_thread = start_loading_thread(Duration::from_secs(60 * 5));
    let websocket_thread = start_websockets(9039);
    
    loading_thread.join().unwrap();
    websocket_thread.join().unwrap();
}
