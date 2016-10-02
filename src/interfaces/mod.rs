use super::*;

mod websocket;

pub trait Interface : Sync {
    fn start(&self, config: &Json) -> Option<thread::JoinHandle<()>>;
}

fn find_interface(name: &str) -> &'static Interface {
    match name {
        "websocket" => websocket::INTERFACE,
        _ => panic!("Cannot find {} interface.", name),
    }
}

#[derive(Deserialize)]
struct BaseSettings {
    enabled: bool,
}

pub fn start_interfaces(config: &Map<Json>) -> Map<Option<thread::JoinHandle<()>>> {
    let mut map = Map::new();
    for (name, config) in config {
        let settings: BaseSettings = serde_json::from_value(config.clone()).unwrap();
        if settings.enabled == false {
            continue
        }
        map.insert(name.clone(), find_interface(&name).start(config));
    }
    map
}
