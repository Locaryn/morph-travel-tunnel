//! Stdio MCP server shipped by plugin-travel-tunnel.
use locaryn_plugin_remote::{
    list_providers, start_remote_tunnel, stop_remote_tunnel, tunnel_status, TunnelRequest,
};
use serde_json::{json, Value};
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle_request(request).await,
            Err(error) => error_response(Value::Null, -32700, format!("JSON invalide : {error}")),
        };
        if let Ok(serialized) = serde_json::to_string(&response) {
            println!("{serialized}");
            let _ = std::io::stdout().flush();
        }
    }
}

async fn handle_request(request: Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match method {
        "initialize" => success(
            id,
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "plugin-travel-tunnel", "version": VERSION }
            }),
        ),
        "tools/list" => success(id, tools_list()),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match call_tool(name, args).await {
                Ok(value) => success(id, text_content(value)),
                Err(error) => error_response(id, -32000, error),
            }
        }
        notification if notification.starts_with("notifications/") => Value::Null,
        _ => error_response(id, -32601, format!("méthode MCP inconnue : {method}")),
    }
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "list_providers",
                "description": "Les relais de tunnel connus, et lesquels sont installés ici.                                 `needs_account` dit lequel exige une inscription avant de servir ;                                 `install_hint` dit comment obtenir celui qui manque.",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "start_remote_tunnel",
                "description": "Ouvre un tunnel sortant vers un port local et rend l'adresse                                 publique que le relais annonce. Un seul tunnel à la fois.                                 L'ouverture peut prendre jusqu'à une minute : le premier                                 lancement d'un relais est lent.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "port": { "type": "integer", "description": "Port local à exposer" },
                        "provider": {
                            "type": "string",
                            "description": "cloudflare, ngrok ou devtunnel. Omis : le premier relais installé, Cloudflare en tête puisqu'il ne demande pas de compte."
                        }
                    },
                    "required": ["port"]
                }
            },
            {
                "name": "tunnel_status",
                "description": "L'état du tunnel, sans rien ouvrir ni fermer.",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "stop_remote_tunnel",
                "description": "Ferme le tunnel. Fermer quand il n'y en a pas n'est pas une erreur.",
                "inputSchema": { "type": "object", "properties": {} }
            }
        ]
    })
}

async fn call_tool(name: &str, args: Value) -> Result<Value, String> {
    match name {
        "list_providers" => Ok(json!({ "providers": list_providers() })),
        "tunnel_status" => Ok(json!(tunnel_status().await)),
        "stop_remote_tunnel" => Ok(json!(stop_remote_tunnel().await)),
        "start_remote_tunnel" => {
            let req: TunnelRequest = serde_json::from_value(args)
                .map_err(|e| format!("Paramètres de tunnel invalides : {e}"))?;
            Ok(json!(start_remote_tunnel(req).await?))
        }
        _ => Err(format!("Outil tunnel inconnu : {name}")),
    }
}

fn text_content(value: Value) -> Value {
    json!({ "content": [{ "type": "text", "text": serde_json::to_string(&value).unwrap_or_else(|_| "{}".into()) }] })
}
fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}
fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
