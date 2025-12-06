use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DHTValue {
    pub key: String,
    pub value: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct WebDHT {
    map: Arc<DashMap<String, DHTValue>>,
}

impl WebDHT {
    pub fn new() -> Self { Self { map: Arc::new(DashMap::new()) } }
    pub fn put(&self, key: String, value: serde_json::Value) {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
        self.map.insert(key.clone(), DHTValue { key, value, timestamp: ts });
    }
    pub fn get(&self, key: &str) -> Option<DHTValue> { self.map.get(key).map(|v| v.clone()) }
    pub fn delete(&self, key: &str) { self.map.remove(key); }
    pub fn all(&self) -> Vec<DHTValue> { self.map.iter().map(|v| v.clone()).collect() }
}

