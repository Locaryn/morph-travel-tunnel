//! Locaryn Remote Mode & Tunneling Plugin
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRequest {
    pub provider: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub active: bool,
    pub public_url: Option<String>,
    pub qr_code_data: Option<String>,
}

/// Non implemente. La signature est conservee pour que l'interface et le
/// serveur MCP gardent leur forme, mais l'appel echoue franchement plutot
/// que de fabriquer un resultat.
pub async fn start_remote_tunnel(_req: TunnelRequest) -> Result<TunnelStatus, String> {
    Err("L'ouverture du tunnel n'est pas implementee : ce morph ne joint aucun service. L'URL publique renvoyee auparavant etait inventee.".into())
}
