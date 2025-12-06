use peerpigeon_rs::PeerPigeonMesh;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut mesh = PeerPigeonMesh::new("global".to_string(), 0, 10);
    mesh.init().await;
    let url = std::env::args().nth(1).unwrap_or_else(|| "ws://localhost:3000".to_string());
    mesh.connect(&url).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("status: {}", mesh.get_status());
    Ok(())
}

