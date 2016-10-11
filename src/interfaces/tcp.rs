use super::super::*;

use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener};
use std::sync::mpsc;

pub static INTERFACE: &'static Interface = &TcpInterface;

struct TcpInterface;

#[derive(Deserialize)]
struct Settings {
    port: u16,
}

impl Interface for TcpInterface {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>{
        use std::thread;
        
        let settings: Settings = serde_json::from_value(config.clone()).unwrap();

        let server = TcpListener::bind(("0.0.0.0", settings.port)).unwrap();
        
        Some(thread::spawn(move || {
            for connection in server.incoming() {
                thread::spawn(move || {
                    let connection = connection.unwrap();
                    let mut sender = connection.try_clone().unwrap();
                    let (tx, rx) = mpsc::channel::<Json>();
                    thread::spawn(move || {
                        let reader = BufReader::new(connection.try_clone().unwrap());
                        for line in reader.lines() {
                            let line = line.unwrap();
                            if line == "" {
                                continue;
                            }
                            let message = serde_json::from_str(&line).unwrap();
                            let response = handle_request(message, &tx)
                                    .unwrap_or_else(|err| response_from_err(err));
                            tx.send(response).unwrap();
                        }
                    });
                    for message in rx {
                        let response_payload = serde_json::to_string(&message).unwrap() + "\n";
                        sender.write_all(response_payload.as_bytes()).unwrap();
                    }
                });
            }
        }))
    }
}
