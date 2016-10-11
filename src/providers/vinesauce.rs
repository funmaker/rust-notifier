use super::super::*;
pub static PROVIDER: &'static Provider = &VinesauceProvider;

struct VinesauceProvider;

#[derive(Deserialize)]
struct Data {
    channels: Vec<String>,
}

#[derive(Deserialize)]
struct Stream {
    preview: Map<String>,
    channel: Channel,
    created_at: String,
}

#[derive(Deserialize)]
struct Channel {
    status: String,
    display_name: String,
}

#[derive(Serialize)]
struct Extra {
    #[serde(rename="displayName")]
    display_name: String,
}

impl Provider for VinesauceProvider {
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>> {
        let mut feed = Feed::new();
        let data: Data = try!(serde_json::from_value(data.clone()));
        
        let team: Json = try!(serde_json::from_slice(&try!(http_get("http://vinesauce.com/twitch/team-data.json"))));
        
        for channel in data.channels {
            if let Some(stream) = team.pointer(&format!("/{}/stream/streams/0", channel)) {
                let stream: Stream = try!(serde_json::from_value(stream.clone()));
                feed.status.push(Entry::new(&stream.channel.status, &hash(&(&stream.created_at, "wonziu")))
                        .set_timestamp(time::strptime(&stream.created_at, "%Y-%m-%dT%H:%M:%SZ")
                                .ok()
                                .map(|tm| to_timestamp(tm)))
                        .link("http://vinesauce.com/")
                        .image_url(stream.preview.get("medium").unwrap())
                        .extra(serde_json::to_value(&Extra{
                                display_name: stream.channel.display_name.to_string(),
                            })))
            }
        }
        
        Ok(feed)
    }
}
