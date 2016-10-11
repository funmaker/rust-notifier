#![feature(custom_derive)]
#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![plugin(serde_macros)]

pub extern crate serde_json;
pub extern crate time;
#[macro_use] extern crate lazy_static;

pub use std::sync::{Arc, Mutex, MutexGuard};
pub use std::error::Error;
pub use std::time::{Duration, Instant};
pub use std::thread;
pub use serde_json::Value as Json;
mod utils;
mod feed;
mod providers;
mod handler;
mod interfaces;
mod config;
mod update;
pub use utils::*;
pub use feed::*;
pub use providers::*;
pub use handler::*;
pub use interfaces::*;
pub use config::*;
pub use update::*;

lazy_static! {
    static ref FEEDS: Mutex<Feeds> = Mutex::new(Feeds::new());
}

pub fn get_feeds() -> MutexGuard<'static, Feeds> {
    FEEDS.lock().unwrap()
}

fn main() {
    let config = load_config().unwrap();
    
    let providers = start_providers(&config.providers);
    let interfaces = start_interfaces(&config.interfaces);
    
    let fetch_thread = start_fetch_thread(Duration::from_secs(60 * 5));
    
    fetch_thread.join().unwrap();
    for (_, thread) in providers {
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }
    for (_, thread) in interfaces {
        if let Some(thread) = thread {
            thread.join().unwrap();
        }
    }
}
