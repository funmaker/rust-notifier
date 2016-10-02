use super::*;
use std::fs::File;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub feeds: Map<ConfigFeedEntry>,
    pub providers: Map<Json>,
    pub interfaces: Map<Json>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigFeedEntry{
    pub provider: String,
    #[serde(rename="providerData")]
    pub provider_data: Json,
    pub color: Option<String>
}

pub fn load_config() -> Result<Config, Box<Error>> {
    let config = try!(File::open("config.json"));
    let config: Config = try!(serde_json::from_reader(config));
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<Error>> {
    use std::io::Write;
    let mut file = try!(File::create("config.json"));
    try!(file.write_all(try!(serde_json::to_string_pretty(config)).as_bytes()));
    Ok(())
}
