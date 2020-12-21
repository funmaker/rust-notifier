use std::cmp::Ordering;
use serde::Serialize;
use chrono::{DateTime, Utc, TimeZone};
use chrono::serde::ts_milliseconds_option;

use crate::utils::Json;

#[derive(Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Entry {
	pub title: String,
	pub guid: String,
	#[serde(rename="feedName")]
	pub feed_name: Option<String>,
	pub description: Option<String>,
	pub link: Option<String>,
	pub color: Option<String>,
	#[serde(rename="imageURL")]
	pub image_url: Option<String>,
	#[serde(with = "ts_milliseconds_option")]
	pub timestamp: Option<DateTime<Utc>>,
	pub extra: Option<Json>,
}

#[allow(dead_code)]
impl Entry {
	pub fn new(title: &str, guid: &str) -> Entry {
		Entry {
			title: title.to_string(),
			guid: guid.to_string(),
			feed_name: None,
			description: None,
			link: None,
			color: None,
			image_url: None,
			timestamp: None,
			extra: None,
		}
	}
	
	pub fn feed_name(mut self, feed_name: &str) -> Entry {
		self.feed_name = Some(feed_name.to_string());
		self
	}
	
	pub fn description(mut self, description: &str) -> Entry {
		self.description = Some(description.to_string());
		self
	}
	
	pub fn link(mut self, link: &str) -> Entry {
		self.link = Some(link.to_string());
		self
	}
	
	pub fn color(mut self, color: &str) -> Entry {
		self.color = Some(color.to_string());
		self
	}
	
	pub fn image_url(mut self, image_url: &str) -> Entry {
		self.image_url = Some(image_url.to_string());
		self
	}
	
	pub fn timestamp<Tz: TimeZone>(mut self, timestamp: DateTime<Tz>) -> Entry {
		self.timestamp = Some(timestamp.with_timezone(&Utc));
		self
	}
	
	pub fn extra(mut self, extra: Json) -> Entry {
		self.extra = Some(extra);
		self
	}
	
	pub fn set_feed_name(mut self, feed_name: Option<String>) -> Entry {
		self.feed_name = feed_name;
		self
	}
	
	pub fn set_description(mut self, description: Option<String>) -> Entry {
		self.description = description;
		self
	}
	
	pub fn set_link(mut self, link: Option<String>) -> Entry {
		self.link = link;
		self
	}
	
	pub fn set_color(mut self, color: Option<String>) -> Entry {
		self.color = color;
		self
	}
	
	pub fn set_image_url(mut self, image_url: Option<String>) -> Entry {
		self.image_url = image_url;
		self
	}
	
	pub fn set_timestamp<Tz: TimeZone>(mut self, timestamp: Option<DateTime<Tz>>) -> Entry {
		self.timestamp = timestamp.map(|ts| ts.with_timezone(&Utc));
		self
	}
	
	pub fn set_extra(mut self, extra: Option<Json>) -> Entry {
		self.extra = extra;
		self
	}
}

// Newest to oldest
impl Ord for Entry {
	fn cmp(&self, other: &Self) -> Ordering {
		match (&self.timestamp, &other.timestamp) {
			(None,    None   ) => Ordering::Equal,
			(Some(_), None   ) => Ordering::Greater,
			(None,    Some(_)) => Ordering::Less,
			(Some(s), Some(o)) => o.cmp(s),
		}
	}
}

impl PartialOrd for Entry {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
