use crate::peer::PeerConnection;
use crate::logger::DebugLogger;
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GossipManager {
    log: DebugLogger,
}

impl GossipManager {
    pub fn new() -> Self { Self { log: DebugLogger::create("Gossip") } }
    pub fn broadcast(&self, peers: &HashMap<String, PeerConnection>, message: serde_json::Value) -> bool {
        let mut ok = true;
        for (pid, pc) in peers.iter() {
            if !pc.send_json(message.clone()) { ok = false; }
        }
        ok
    }
}

