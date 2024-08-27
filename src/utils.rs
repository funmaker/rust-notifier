use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use futures::stream;
use futures::stream::Iter;
use itertools::Itertools;

use crate::feeds::Feed;

pub use serde_json::Value as Json;
pub type Map<T> = BTreeMap<String, T>;

pub fn hash<H: Hash>(data: &H) -> String {
	let mut state = DefaultHasher::new();
	data.hash(&mut state);
	format!("{:X}", state.finish())
}

pub trait IteratorEx: Iterator + Sized {
	fn into_stream(self) -> Iter<Self> {
		stream::iter(self)
	}
	
	fn into_box<'a>(self)
	               -> Box<dyn Iterator<Item=Self::Item> + 'a>
	               where Self: 'a {
		Box::new(self)
	}
	
	fn kmerge_feeds<'a>(self) -> Feed
	                          where Self: Iterator<Item = &'a Feed> + Clone {
		let mut feed = Feed::new();
		
		feed.status = self.clone().map(|feed| &feed.status).cloned().kmerge().collect();
		feed.notifications = self.clone().map(|feed| &feed.notifications).cloned().kmerge().collect();
		feed.errors = self.clone().map(|feed| &feed.errors).cloned().kmerge().collect();
		
		feed
	}
}

impl<I> IteratorEx for I where I: Iterator {}
