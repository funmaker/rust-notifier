use super::super::*;

extern crate websocket;
use self::websocket::{Server, Message, Receiver, Sender};
use std::sync::mpsc;

pub static INTERFACE: &'static Interface = &WebSocketInterface;

struct WebSocketInterface;

#[derive(Deserialize)]
struct Settings {
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
                    let (tx, rx) = mpsc::channel::<Json>();
                    thread::spawn(move || {
                        for message in receiver.incoming_messages() {
                            let message: Message = message.unwrap();
                            match message.opcode {
                                Ping => {
                                    //message.into_pong().unwrap();
                                    //ping_sender.send_message(&message).unwrap()
                                },
                                Pong => {},
                                Close => break,
                                Text | Binary => {
                                    let response = handle_message(message, &tx)
                                            .unwrap_or_else(|err| response_from_err(err));
                                    tx.send(response).unwrap()
                                }
                            }
                        }
                    });
                    for message in rx {
                        let response_payload = serde_json::to_string(&message).unwrap();
                        let response_message = Message::text(response_payload);
                        sender.send_message(&response_message).unwrap();
                    }
                });
            }
        }))
    }
}

fn handle_message(message: Message, tx: &mpsc::Sender<Json>) -> Result<Json, Box<Error>> {
    let message = message.payload;
    let request = try!(serde_json::from_slice(&message));
    handle_request(request, tx)
}
