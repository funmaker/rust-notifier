use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::pin::Pin;
use serde::{Serialize, Deserialize};
use futures::stream;
use futures::stream::Iter;

pub use serde_json::Value as Json;

pub type Map<T> = BTreeMap<String, T>;

#[derive(Serialize, Deserialize, Hash, Debug, Copy, Clone)]
pub struct Timestamp(u128);

pub fn hash<H: Hash>(data: &H) -> String {
	let mut state = DefaultHasher::new();
	data.hash(&mut state);
	format!("{:X}", state.finish())
}

pub trait IteratorEx: Iterator + Sized {
	fn into_stream(self) -> Iter<Self> {
		stream::iter(self)
	}
	
	fn into_box<'a>(self) -> Box<dyn Iterator<Item=Self::Item> + 'a>
	                where Self: 'a {
		Box::new(self)
	}
	
	fn into_pin<'a>(self) -> Pin<Box<dyn Iterator<Item=Self::Item> + 'a>>
	                      where Self: 'a {
		Box::pin(self)
	}
}

impl<I> IteratorEx for I where I: Iterator {}
