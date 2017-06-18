use super::*;
use std::collections::{BTreeSet, BTreeMap};
use std::sync::mpsc;

lazy_static! {
    static ref HASHES: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
    static ref UPDATERS: Mutex<Vec<mpsc::Sender<Json>>> = Mutex::new(Vec::new());
}

pub fn run_update(feeds: &Feeds) {
    let mut hashes = HASHES.lock().unwrap();
    let mut updaters = UPDATERS.lock().unwrap();

    let response = UpdateResponse {
        status: feeds.values()
            .flat_map(|f| f.status.iter())
            .filter(|e| !hashes.contains(&e.guid))
            .collect(),
        notifications: feeds.values()
            .flat_map(|f| f.notifications.iter())
            .filter(|e| !hashes.contains(&e.guid))
            .collect(),
    };

    if response.status.len() == 0 && response.notifications.len() == 0 {
        return;
    }

    let mut response = serde_json::to_value(response).unwrap();

    response.as_object_mut().unwrap().insert("command".to_string(), Json::String("update".to_string()));

    hashes.clear();
    hashes.extend(feeds.values()
            .flat_map(|f| f.iter())
            .map(|e| e.guid.to_string()));

    updaters.retain(|tx| tx.send(response.clone()).is_ok());
}

pub fn add_updater(tx: &mpsc::Sender<Json>) {
    UPDATERS.lock().unwrap().push(tx.clone());
}
