extern crate websocket;

use super::super::*;
use self::websocket::{ClientBuilder, Message};

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
            let client = ClientBuilder::new("wss://api.pancernik.info/notifier")
                .unwrap()
                .connect_insecure()
                .unwrap();
            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let mut message: Message = Message::from(message.unwrap());
                match message.opcode {
                    Text | Binary => {
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
    data: Option<Json>,
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
            *STATUS.lock().unwrap() = serde_json::from_value(message.data.unwrap()).unwrap();
            None
        }
        "update" => {
            let status = serde_json::to_value(&*STATUS.lock().unwrap()).unwrap();
            let status_new = message.data.unwrap();
            *STATUS.lock().unwrap() = serde_json::from_value(json_merge(status, status_new)).unwrap();
            None
        }
        "ping" => Some(Message::text(serde_json::to_string(&WonziuMessage{kind: "pong".to_string(), data:None}).unwrap())),
        _ => unreachable!(),
    }
}

fn json_merge(obj: Json, diff: Json) -> Json {
    match diff {
        Json::Array(diff) => {
                if let Json::Array(mut obj) = obj {
                    for (n, el) in diff.into_iter().enumerate() {
                        let merged = json_merge(obj[n].clone(), el);
                        obj.insert(n, merged);
                    }
                    Json::Array(obj)
                } else {
                    Json::Array(diff)
                }
            },
        Json::Object(diff) => {
                if let Json::Object(mut obj) = obj {
                    for (k, v) in diff.into_iter() {
                        let merged = json_merge(obj.remove(&k).unwrap_or(Json::Null), v);
                        obj.insert(k, merged);
                    }
                    Json::Object(obj)
                } else {
                    Json::Object(diff)
                }
            },
        _ => diff,
    }
}
