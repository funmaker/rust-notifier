use std::sync::Arc;
use arc_swap::ArcSwap;

use crate::feeds::Feeds;
use crate::config::ConfigFeedEntry;
use crate::utils::Map;

#[derive(Debug, Clone)]
pub struct State {
	pub feed_entries: Arc<ArcSwap<Map<ConfigFeedEntry>>>,
	pub feeds: Arc<ArcSwap<Feeds>>,
}

impl State {
	pub(crate) fn new(feed_entries: Map<ConfigFeedEntry>) -> Self {
		State {
			feed_entries: Arc::new(ArcSwap::from_pointee(feed_entries)),
			feeds: Arc::new(ArcSwap::from_pointee(Feeds::new())),
		}
	}
}
