pub extern crate curl;

use super::*;
use std::fmt::{Display, self, Formatter};
use std::hash::*;

pub fn to_timestamp(tm: time::Tm) -> u64 {
    let tm = tm.to_timespec();
    tm.sec as u64 * 1000 + (tm.nsec as u64 / 1000 / 1000)
}

pub fn timestamp() -> u64 {
    to_timestamp(time::now())
}

pub fn http_get(url: &str) -> Result<Vec<u8>, Box<Error>> {
    use curl::easy::Easy;

    let mut data = Vec::new();
    let mut handle = Easy::new();
    try!(handle.url(url));
    {
        let mut transfer = handle.transfer();
        try!(transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }));
        try!(transfer.perform());
    }
    Ok(data)
}

pub fn hash<H: Hash>(data: &H) -> String {
    let mut state = SipHasher::new();
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
