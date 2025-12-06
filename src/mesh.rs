use crate::crypto::CryptoManager;
use crate::dht::WebDHT;
use crate::gossip::GossipManager;
use crate::logger::DebugLogger;
use crate::peer::PeerConnection;
use crate::signaling::{SignalingClient, SignalingEnvelope};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub struct PeerPigeonMesh {
    pub peer_id: String,
    pub network_name: String,
    pub min_peers: usize,
    pub max_peers: usize,
    pub signaling: SignalingClient,
    pub peers: HashMap<String, PeerConnection>,
    pub gossip: GossipManager,
    pub dht: WebDHT,
    pub crypto: CryptoManager,
    log: DebugLogger,
}

impl PeerPigeonMesh {
    pub fn new(network_name: String, min_peers: usize, max_peers: usize) -> Self {
        let peer_id = Uuid::new_v4().to_string();
        let signaling = SignalingClient::new(peer_id.clone(), network_name.clone());
        Self { peer_id, network_name, min_peers, max_peers, signaling, peers: HashMap::new(), gossip: GossipManager::new(), dht: WebDHT::new(), crypto: CryptoManager::new(), log: DebugLogger::create("Mesh") }
    }
    pub async fn init(&mut self) {}
    pub async fn connect(&mut self, ws_url: &str) -> anyhow::Result<()> {
        self.signaling.connect(ws_url).await?;
        let mut rx = self.signaling.events.subscribe();
        let pid = self.peer_id.clone();
        let net = self.network_name.clone();
        let sig = self.signaling.clone();
        tokio::spawn(async move {
            let _ = sig.announce(json!({"features": {"gossip": true, "dht": true}, "network": net})).await;
        });
        let peers = tokio::sync::Mutex::new(HashSet::new());
        let self_id = self.peer_id.clone();
        let sender = self.signaling.clone();
        let peers_map_ptr = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let ice_servers = vec!["stun:stun.l.google.com:19302".to_string()];
        let mesh_ptr = tokio::sync::Mutex::new(());
        let mut_mesh = tokio::sync::Mutex::new(());
        let peers_ref = tokio::sync::Mutex::new(());
        let pc_store = tokio::sync::Mutex::new(()) ;
        let announce_sender = self.signaling.clone();
        let mesh_ptr2 = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let peers_map = std::sync::Arc::new(tokio::sync::Mutex::new(())).clone();
        let mesh_events = self.signaling.events.clone();
        let mut rx2: Receiver<SignalingEnvelope> = self.signaling.events.subscribe();
        let mut_rx = self.signaling.events.subscribe();
        let peers_handle = std::sync::Arc::new(tokio::sync::Mutex::new(0usize));
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                match msg.r#type.as_str() {
                    "peer-discovered" => {
                        if let Some(data) = msg.data.get("peerId").and_then(|v| v.as_str()) {
                            if data != self_id {
                                let should_initiate = self_id < data;
                                if should_initiate {
                                    let ice = ice_servers.clone();
                                    let target = data.to_string();
                                    let sender2 = sender.clone();
                                    tokio::spawn(async move {
                                        if let Ok(pc) = PeerConnection::new(target.clone(), true, ice).await {
                                            let _ = pc.create_data_channel("mesh").await;
                                            if let Ok(offer) = pc.create_offer().await {
                                                let env = SignalingEnvelope { r#type: "offer".into(), data: json!({"type":"offer","sdp": offer.sdp}), targetPeerId: Some(target.clone()), networkName: None, fromPeerId: None, timestamp: None };
                                                let _ = sender2.send(env).await;
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                    "offer" => {
                        if let Some(sdp) = msg.data.get("sdp").and_then(|v| v.as_str()) { 
                            if let Some(from) = msg.fromPeerId.clone() {
                                let ice = ice_servers.clone();
                                let sender2 = sender.clone();
                                tokio::spawn(async move {
                                    if let Ok(pc) = PeerConnection::new(from.clone(), false, ice).await {
                                        let _ = pc.create_data_channel("mesh").await;
                                        let offer = RTCSessionDescription::offer(sdp.to_string());
                                        if let Ok(answer) = pc.handle_offer(offer).await {
                                            let env = SignalingEnvelope { r#type: "answer".into(), data: json!({"type":"answer","sdp": answer.sdp}), targetPeerId: Some(from.clone()), networkName: None, fromPeerId: None, timestamp: None };
                                            let _ = sender2.send(env).await;
                                        }
                                    }
                                });
                            }
                        }
                    }
                    "answer" => {
                        if let Some(sdp) = msg.data.get("sdp").and_then(|v| v.as_str()) { 
                            if let Some(from) = msg.fromPeerId.clone() {
                                let answer = RTCSessionDescription::answer(sdp.to_string());
                                let _ = &answer;
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }
    pub fn get_status(&self) -> serde_json::Value {
        json!({
            "peerId": self.peer_id,
            "networkName": self.network_name,
            "minPeers": self.min_peers,
            "maxPeers": self.max_peers,
            "connectedCount": self.peers.len(),
        })
    }
    pub fn send_message(&self, content: serde_json::Value) -> bool {
        self.gossip.broadcast(&self.peers, json!({"type":"message","content": content}))
    }
}

