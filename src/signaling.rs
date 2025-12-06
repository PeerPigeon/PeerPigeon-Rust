use crate::logger::DebugLogger;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;
use url::Url;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SignalingEnvelope {
    pub r#type: String,
    pub data: serde_json::Value,
    #[serde(default)]
    pub targetPeerId: Option<String>,
    #[serde(default)]
    pub networkName: Option<String>,
    #[serde(default)]
    pub fromPeerId: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

pub struct SignalingClient {
    log: DebugLogger,
    peer_id: String,
    pub network_name: String,
    tx: Option<tokio::sync::mpsc::Sender<SignalingEnvelope>>, 
    pub events: broadcast::Sender<SignalingEnvelope>,
    ws_url: Option<Url>,
}

impl SignalingClient {
    pub fn new(peer_id: String, network_name: String) -> Self {
        let (events, _) = broadcast::channel(256);
        Self { log: DebugLogger::create("Signaling"), peer_id, network_name, tx: None, events, ws_url: None }
    }
    pub async fn connect(&mut self, url: &str) -> anyhow::Result<()> {
        let mut u = Url::parse(url)?;
        u.query_pairs_mut().append_pair("peerId", &self.peer_id);
        self.ws_url = Some(u.clone());
        let (ws, _) = connect_async(u).await?;
        let (mut write, mut read) = ws.split();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<SignalingEnvelope>(256);
        self.tx = Some(tx);
        let events = self.events.clone();
        let net = self.network_name.clone();
        let pid = self.peer_id.clone();
        tokio::spawn(async move {
            while let Some(env) = rx.recv().await {
                let mut payload = serde_json::json!({"type": env.r#type, "data": env.data, "networkName": net});
                if let Some(t) = env.targetPeerId { payload["targetPeerId"] = serde_json::Value::String(t); }
                let _ = write.send(tungstenite::Message::Text(payload.to_string())).await;
            }
        });
        tokio::spawn(async move {
            while let Some(msg) = read.next().await { 
                if let Ok(t) = msg { 
                    match t { 
                        tungstenite::Message::Text(s) => {
                            if let Ok(mut v) = serde_json::from_str::<SignalingEnvelope>(&s) {
                                v.fromPeerId.get_or_insert(pid.clone());
                                let _ = events.send(v);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(())
    }
    pub async fn send(&self, env: SignalingEnvelope) -> anyhow::Result<()> {
        if let Some(tx) = &self.tx { tx.send(env).await.map_err(|e| anyhow::anyhow!(e.to_string()))?; }
        Ok(())
    }
    pub async fn announce(&self, data: serde_json::Value) -> anyhow::Result<()> {
        self.send(SignalingEnvelope { r#type: "announce".into(), data, targetPeerId: None, networkName: Some(self.network_name.clone()), fromPeerId: Some(self.peer_id.clone()), timestamp: None }).await
    }
}

