use std::collections::HashSet;
use chrono::{DateTime, Utc};

mod entry;
pub use entry::Entry;

mod feed;
pub use feed::Feed;
use crate::utils::Map;

#[derive(Debug)]
pub struct Feeds {
	pub created: DateTime<Utc>,
	pub feeds: Map<Feed>,
	pub ids: Option<HashSet<String>>
}

impl Feeds {
	pub fn new() -> Self { Feeds{
		created: Utc::now(),
		feeds: Map::new(),
		ids: None,
	}}
}

impl std::ops::Deref for Feeds {
	type Target = Map<Feed>;
	
	fn deref(&self) -> &Map<Feed> {
		&self.feeds
	}
}

impl std::ops::DerefMut for Feeds {
	fn deref_mut(&mut self) -> &mut Map<Feed> {
		&mut self.feeds
	}
}


