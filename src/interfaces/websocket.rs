use super::super::*;

extern crate websocket;
use self::websocket::{Server, Message, Receiver, Sender, WebSocketStream, Client};
use self::websocket::ws::dataframe::DataFrame;
use self::websocket::result::WebSocketError;

pub static INTERFACE: &'static Interface = &WebSocketInterface;

struct WebSocketInterface;

#[derive(Deserialize)]
struct Settings {
    enabled: bool,
    port: u16,
}

impl Interface for WebSocketInterface {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>{
        use std::thread;
        use self::websocket::message::Type::*;
        
        let settings: Settings = serde_json::from_value(config.clone()).unwrap();

        let server = Server::bind(("0.0.0.0", settings.port)).unwrap();
        
        Some(thread::spawn(move || {
            for connection in server {
                thread::spawn(move || {
                    let (mut sender, mut receiver) = connection.unwrap().read_request().unwrap().accept().send().unwrap().split();
                    for message in receiver.incoming_messages() {
                        let mut message: Message = message.unwrap();
                        match message.opcode {
                            Ping => {
                                message.into_pong().unwrap();
                                sender.send_message(&message).unwrap()
                            },
                            Pong => {},
                            Close => break,
                            Text | Binary => {
                                let response = handle_message(message)
                                        .unwrap_or_else(|err| response_from_err(err));
                                let response_payload = serde_json::to_string(&response).unwrap();
                                let response_message = Message::text(response_payload);
                                sender.send_message(&response_message).unwrap()
                            }
                        }
                    }
                });
            }
        }))
    }
}

fn handle_message(message: Message) -> Result<Json, Box<Error>> {
    let message = message.payload;
    let request = try!(serde_json::from_slice(&message));
    handle_request(request)
}
