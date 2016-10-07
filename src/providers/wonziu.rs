extern crate websocket;

use super::super::*;
use self::websocket::{Client, Message, Receiver, Sender};
use self::websocket::client::request::Url;

pub static PROVIDER: &'static Provider = &WonziuProvider;

struct WonziuProvider;

lazy_static!{
    static ref STATUS: Mutex<WonziuData> = Mutex::new(WonziuData{stream: None, topic: None});
}

impl Provider for WonziuProvider {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>{
        use self::websocket::message::Type::*;
        let config = config.clone();
        Some(thread::spawn(move || {
            let url = Url::parse("wss://api.pancernik.info/notifier").unwrap();
            let request = Client::connect(url).unwrap();
            let response = request.send().unwrap();
            response.validate().unwrap();
            let client = response.begin();
            let (mut sender, mut receiver) = client.split();
            
            for message in receiver.incoming_messages() {
                let mut message: Message = message.unwrap();
                match message.opcode {
                    Text | Binary => {
                        println!("{}", String::from_utf8_lossy(&message.payload));
                        if let Some(msg) = handle_message(serde_json::from_slice(&message.payload).unwrap()) {
                            sender.send_message(&msg).unwrap();
                        }
                    },
                    Ping => {
                        message.into_pong().unwrap();
                        sender.send_message(&message).unwrap();
                    },
                    Pong => {},
                    Close => {PROVIDER.start(&config); panic!("Wonziu websocket closed!")}
                }
            }
        }))
    }
    
    fn load_feed(&self, _data: &Json) -> Result<Feed, Box<Error>> {
        let mut feed = Feed::new();
        
        let stream;
        let topic;
        {
            let status = STATUS.lock().unwrap();
            if status.stream.is_none() {
                return Ok(feed);
            }
            stream = status.stream.clone().unwrap();
            topic = status.topic.clone().unwrap();
        }
        
        if stream.status {
            feed.status.push(Entry::new(&topic.text, &hash(&(&stream.online_at, "wonziu")))
                    .set_timestamp(time::strptime(&stream.online_at, "%Y-%m-%dT%H:%M:%S")
                            .ok()
                            .map(|tm| to_timestamp(tm)))
                    .link("http://jadisco.pl/"))
        }
        
        Ok(feed)
    }
}

#[derive(Deserialize, Serialize)]
struct WonziuMessage {
    #[serde(rename="type")]
    kind: String,
    data: Option<WonziuData>,
}

#[derive(Deserialize, Serialize)]
struct WonziuData {
    stream: Option<WonziuStream>,
    topic: Option<WonziuTopic>,
}

#[derive(Deserialize, Serialize, Clone)]
struct WonziuStream {
    status: bool,
    online_at: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct WonziuTopic {
    text: String,
}

fn handle_message(message: WonziuMessage) -> Option<Message<'static>> {
    match &message.kind[..] {
        "status" => {
            *STATUS.lock().unwrap() = message.data.unwrap();
            None
        }
        "update" => {
            let status = message.data.unwrap();
            if let Some(stream) = status.stream {
                STATUS.lock().unwrap().stream = Some(stream);
            }
            if let Some(topic) = status.topic {
                STATUS.lock().unwrap().topic = Some(topic);
            }
            None
        }
        "ping" => Some(Message::text(serde_json::to_string(&WonziuMessage{kind: "pong".to_string(), data:None}).unwrap())),
        _ => unreachable!(),
    }
}
