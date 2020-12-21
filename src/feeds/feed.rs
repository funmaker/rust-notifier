use serde::Serialize;
use chrono::Utc;

use super::Entry;
use crate::utils::hash;

#[derive(Serialize, Clone, Debug)]
pub struct Feed {
	pub status: Vec<Entry>,
	pub notifications: Vec<Entry>,
	pub errors: Vec<Entry>,
}

#[allow(dead_code)]
impl Feed {
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entry> {
		self.status.iter_mut().chain(self.errors.iter_mut()).chain(self.notifications.iter_mut())
	}
	
	pub fn iter(&self) -> impl Iterator<Item = &Entry> {
		self.status.iter().chain(self.errors.iter()).chain(self.notifications.iter())
	}
	
	pub fn new() -> Self {
		Feed {
			status: Vec::new(),
			notifications: Vec::new(),
			errors: Vec::new(),
		}
	}
	
	pub fn append(mut self, mut other: Feed) -> Self {
		self.status.append(&mut other.status);
		self.notifications.append(&mut other.notifications);
		self.errors.append(&mut other.errors);
		self
	}
	
	pub fn from_err(err: &str, desc: &str) -> Self {
		let mut feed = Feed::new();
		feed.add_err(err, desc);
		feed
	}
	
	pub fn add_err(&mut self, err: &str, desc: &str) {
		self.errors.push(
			Entry::new(err, &hash(&(Utc::now(), err, desc)))
				.description(desc)
				.color("#FF0000")
				.timestamp(Utc::now()),
		);
	}
}
