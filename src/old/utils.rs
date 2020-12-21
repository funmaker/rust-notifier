pub extern crate curl;

use super::*;
use std::fmt::{Display, self, Formatter};
use std::hash::*;
use std::collections::hash_map::DefaultHasher;
use std::cmp::Ordering;

pub fn to_timestamp(tm: time::Tm) -> u64 {
    let tm = tm.to_timespec();
    tm.sec as u64 * 1000 + (tm.nsec as u64 / 1000 / 1000)
}

pub fn from_timestamp(ts: u64) -> time::Tm {
    time::at_utc(time::Timespec::new((ts / 1000) as i64, (ts % 1000) as i32))
}

pub fn timestamp() -> u64 {
    to_timestamp(time::now())
}

pub fn http_get(url: &str) -> Result<Vec<u8>, Box<Error>> {
    use curl::easy::Easy;

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(url)?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }
    Ok(data)
}

pub fn hash<H: Hash>(data: &H) -> String {
    let mut state = DefaultHasher::new();
    data.hash(&mut state);
    format!("{:X}", state.finish())
}

#[derive(Debug)]
pub struct HandleError(String);

impl Error for HandleError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl HandleError {
    pub fn new<T>(text: String) -> Result<T, Box<Error>> {
        Err(Box::new(HandleError(text)))
    }
}

impl Display for HandleError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}


pub trait Sorted where Self: Sized {
    type Item;
    fn sorted_by<F>(&mut self, compare: F) -> Vec<Self::Item>
        where F: FnMut(&Self::Item, &Self::Item) -> Ordering;
}

impl<T> Sorted for T where T: Iterator {
    type Item = T::Item;

    fn sorted_by<F>(&mut self, compare: F) -> Vec<Self::Item>
        where F: FnMut(&Self::Item, &Self::Item) -> Ordering {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by(compare);
        vec
    }
}
