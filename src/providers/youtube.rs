extern crate rss;
extern crate url;

use super::super::*;
pub static PROVIDER: &'static Provider = &YTProvider;
use self::url::percent_encoding::percent_encode;
use self::url::percent_encoding::QUERY_ENCODE_SET;

struct YTProvider;

lazy_static! {
    static ref API_KEY: Mutex<String> = Mutex::new(String::new());
}

macro_rules! ytcall(
    { $api_key:expr, $command:expr, $($key:expr => $value:expr),+ } => {
        {
            let mut s = format!("https://www.googleapis.com/youtube/v3/{}?key={}", $command, $api_key);
            $(
                s += &format!("&{}={}", percent_encode($key.as_bytes(), QUERY_ENCODE_SET), percent_encode($value.to_string().as_bytes(), QUERY_ENCODE_SET));
            )*
            let response = try!(http_get(&s));
            try!(serde_json::from_slice::<YouTubeResponse>(&response))
        }
     };
);

#[derive(Deserialize)]
struct Settings {
    #[serde(rename="apiKey")]
    api_key: String,
}

#[derive(Deserialize)]
struct YouTubeResponse {
    #[serde(rename="nextPageToken")]
    next_page_token: Option<String>,
    items: Vec<Json>,
}

#[derive(Deserialize, Debug)]
struct Video {
    #[serde(rename="publishedAt")]
    published_at: String,
    title: String,
    #[serde(rename="channelTitle")]
    channel_title: String,
    thumbnails: Json,
    id: Option<String>,
}

impl Video {
    fn to_entry(self) -> Option<Entry> {
        match (self.thumbnails.pointer("/default/url"), self.id) {
            (Some(&Json::String(ref tbnail)), Some(ref id)) => {
                Some(Entry::new(&self.title, id)
                        .link(&format!("https://youtu.be/{}", id))
                        .set_timestamp(time::strptime(&self.published_at, "%Y-%m-%dT%H:%M:%S")
                                .ok()
                                .map(|tm| to_timestamp(tm)))
                        .feed_name(&self.channel_title)
                        .image_url(tbnail))
            }
            _ => None
        }
    }
    
    fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
}

impl Provider for YTProvider {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>{
        let settings: Settings = serde_json::from_value(config.clone()).unwrap();
        *API_KEY.lock().unwrap() = settings.api_key;
        None
    }
    
    fn load_feed(&self, data: &Json) -> Result<Feed, Box<Error>> {
        let channel: String = try!(serde_json::from_value(data.clone()));
        let api_key = API_KEY.lock().unwrap().clone();
        
        let mut all_playlists = Vec::new();
        
        let mut subscriptions = ytcall![api_key, "subscriptions",
                "part" => "snippet",
                "channelId" => channel,
                "maxResults" => 50 ];
        loop {
            let channels = subscriptions.items.iter()
                    .filter_map(|subscription| subscription.pointer("/snippet/resourceId/channelId").and_then(|c| c.as_str()))
                    .fold(String::new(), |mut acc, s| {acc += s; acc += ","; acc});
            
            let playlists = ytcall![api_key, "channels",
                    "part" => "contentDetails",
                    "id" => channels,
                    "maxResults" => 50 ];
            
            all_playlists.extend(playlists.items.iter()
                    .filter_map(|playlist| playlist.pointer("/contentDetails/relatedPlaylists/uploads")
                            .and_then(|u| u.as_str()
                                    .map(|s| s.to_string()))));
            
            if let Some(next_page) = subscriptions.next_page_token {
                subscriptions = ytcall![api_key, "subscriptions",
                        "part" => "snippet",
                        "channelId" => channel,
                        "maxResults" => 50,
                        "pageToken" => next_page ];
            } else {
                break;
            }
        }
        
        let mut all_plitems = Vec::new();
        
        for playlist in all_playlists {
            let plitems = ytcall![api_key, "playlistItems",
                    "part" => "contentDetails",
                    "playlistId" => playlist,
                    "maxResults" => 5 ];
                    
            all_plitems.extend(plitems.items.iter()
                    .filter_map(|plitem| plitem.pointer("/contentDetails/videoId")
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string())));
        }
        
        let mut feed = Feed::new();
        
        for plitems in all_plitems.chunks(50) {
            let ids = plitems.join(",");
                    
            let videos = ytcall![api_key, "videos",
                    "part" => "snippet",
                    "id" => ids,
                    "maxResults" => 50 ];
                    
            feed.notifications.extend(videos.items.iter()
                    .filter_map(|video| video.pointer("/id")
                            .and_then(|id| id.as_str())
                            .and_then(|id| video.pointer("/snippet")
                                    .map(|snippet| (id, snippet)))
                            .and_then(|(id, snippet)| serde_json::from_value::<Video>(snippet.clone())
                                    .ok()
                                    .map(|video| video.id(id))))
                    .filter_map(|video| video.to_entry()));
        }
        
        Ok(feed)
    }
}