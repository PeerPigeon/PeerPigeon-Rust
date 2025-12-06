use crate::logger::DebugLogger;
use futures::StreamExt;
use serde_json::Value;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtc_ice_server::RTCIceServer;
use std::sync::Arc;

pub struct PeerConnection {
    pub peer_id: String,
    pub is_initiator: bool,
    pub pc: Arc<RTCPeerConnection>,
    pub dc: Option<Arc<RTCDataChannel>>,
    log: DebugLogger,
}

impl PeerConnection {
    pub async fn new(peer_id: String, is_initiator: bool, ice_servers: Vec<String>) -> anyhow::Result<Self> {
        let mut m = MediaEngine::default();
        m.register_default_codecs()?;
        let api = APIBuilder::new().with_media_engine(m).build();
        let cfg = RTCConfiguration { ice_servers: vec![RTCIceServer { urls: ice_servers, ..Default::default() }], ..Default::default() };
        let pc = Arc::new(api.new_peer_connection(cfg).await?);
        let log = DebugLogger::create("PeerConnection");
        let mut this = Self { peer_id, is_initiator, pc: pc.clone(), dc: None, log };
        let pc2 = pc.clone();
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let _ = &pc2;
            println!("PeerConnection: state={:?}", s);
            Box::pin(async {})
        }));
        Ok(this)
    }
    pub async fn create_data_channel(&mut self, label: &str) -> anyhow::Result<()> {
        let dc = self.pc.create_data_channel(label, None).await?;
        let dc = Arc::new(dc);
        dc.on_open(Box::new(move || { Box::pin(async {}) }));
        Ok({ self.dc = Some(dc); () })
    }
    pub async fn create_offer(&self) -> anyhow::Result<RTCSessionDescription> {
        let offer = self.pc.create_offer(None).await?;
        self.pc.set_local_description(offer.clone()).await?;
        Ok(offer)
    }
    pub async fn handle_offer(&self, offer: RTCSessionDescription) -> anyhow::Result<RTCSessionDescription> {
        self.pc.set_remote_description(offer).await?;
        let answer = self.pc.create_answer(None).await?;
        self.pc.set_local_description(answer.clone()).await?;
        Ok(answer)
    }
    pub async fn handle_answer(&self, answer: RTCSessionDescription) -> anyhow::Result<()> {
        self.pc.set_remote_description(answer).await?;
        Ok(())
    }
    pub async fn add_ice_candidate(&self, cand: RTCIceCandidate) -> anyhow::Result<()> {
        self.pc.add_ice_candidate(cand).await?;
        Ok(())
    }
    pub fn send_json(&self, v: Value) -> bool {
        if let Some(dc) = &self.dc { let s = v.to_string(); if let Err(_) = dc.send_text(s) { return false; } return true; }
        false
    }
}

