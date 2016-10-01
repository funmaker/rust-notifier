use super::*;

extern crate websocket;
use self::websocket::{Server, Message, Receiver, Sender, WebSocketStream, Client};
use self::websocket::ws::dataframe::DataFrame;
use self::websocket::result::WebSocketError;

pub fn start_websockets(port: u16) -> thread::JoinHandle<()> {
    use std::thread;
    use self::websocket::message::Type::*;

    let server = Server::bind(("127.0.0.1", port)).unwrap();
    
    thread::spawn(move || {
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
                                    .unwrap_or_else(|err|
                                            serde_json::to_value(&Feed::from_err("Error in handling request.", err.description())) );
                            let response_payload = serde_json::to_string(&response).unwrap();
                            let response_message = Message::text(response_payload);
                            sender.send_message(&response_message).unwrap()
                        }
                    }
                }
            });
        }
    })
}

fn handle_message(message: Message) -> Result<serde_json::Value, Box<Error>> {
    let message = message.payload;
    let request = try!(serde_json::from_slice(&message));
    let response = handle_request(request);
    Ok(response)
}