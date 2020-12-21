use super::*;
use std::fs::File;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub feeds: Map<ConfigFeedEntry>,
    pub providers: Map<Json>,
    pub interfaces: Map<Json>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigFeedEntry {
    pub provider: String,
    #[serde(rename="providerData")]
    pub provider_data: Json,
    pub color: Option<String>
}

pub fn load_config() -> Result<Config, Box<Error>> {
    match File::open("config.json") {
        Ok(file) => {
            let config: Config = serde_json::from_reader(file)?;
            Ok(config)
        },
        Err(_) => {
            println!("config.json not found.\nGenerating from config_example.json.");
            let file = File::open("config_example.json")?;
            let config: Config = serde_json::from_reader(file)?;
            save_config(&config)?;
            Ok(config)
        },
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<Error>> {
    use std::io::Write;
    let mut file = File::create("config.json")?;
    file.write_all(serde_json::to_string_pretty(config)?.as_bytes())?;
    Ok(())
}
