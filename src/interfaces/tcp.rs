use super::super::*;

use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener};

pub static INTERFACE: &'static Interface = &TcpInterface;

struct TcpInterface;

#[derive(Deserialize)]
struct Settings {
    enabled: bool,
    port: u16,
}

impl Interface for TcpInterface {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>{
        use std::thread;
        
        let settings: Settings = serde_json::from_value(config.clone()).unwrap();

        let server = TcpListener::bind(("127.0.0.1", settings.port)).unwrap();
        
        Some(thread::spawn(move || {
            for connection in server.incoming() {
                thread::spawn(move || {
                    let mut connection = connection.unwrap();
                    let reader = BufReader::new(connection.try_clone().unwrap());
                    for line in reader.lines() {
                        let line = line.unwrap();
                        if line == "" {
                            continue;
                        }
                        let message = serde_json::from_str(&line).unwrap();
                        let response = handle_request(message)
                                .unwrap_or_else(|err| response_from_err(err));
                        let response_payload = serde_json::to_string(&response).unwrap();
                        connection.write_all(response_payload.as_bytes()).unwrap();
                    }
                });
            }
        }))
    }
}
