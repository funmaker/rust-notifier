use serde::Serialize;
use chrono::Utc;
use anyhow::Error;
use serde_json::json;
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
	
	pub fn from_err<'d, 'e>(desc: &'d str, err: &'e Error) -> Self {
		let mut feed = Feed::new();
		feed.add_err(desc, err);
		feed
	}
	
	pub fn add_err<'s, 'd, 'e>(&'s mut self, desc: &'d str, err: &'e Error) {
		let message = err.to_string();
		
		let stack = err.backtrace().to_string();
		
		let extra = if let Some(err) = err.downcast_ref::<reqwest::Error>() {
			json!({
				"url": err.url().map(|url| url.as_str()),
				"status": err.status().map(|url| url.as_u16()),
				"stack": stack,
			})
		} else {
			json!({
				"stack": stack,
			})
		};
		
		self.errors.push(
			Entry::new(desc, &hash(&(Utc::now(), desc, &message)))
				.description(&message)
				.set_link(err.downcast_ref().and_then(|err: &reqwest::Error| err.url().map(|url| url.to_string())))
				.color("#FF0000")
				.timestamp(Utc::now())
				.extra(extra)
		);
	}
}
