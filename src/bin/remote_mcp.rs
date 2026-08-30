//! Stdio MCP server shipped by plugin-travel-tunnel.
use locaryn_plugin_remote::list_providers;
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

/// Ce que ce serveur sait dire.
///
/// Ouvrir et fermer un tunnel n'en font plus partie. Le service local en tient
/// deja un, et c'est le sien que porte le code d'appairage : un tunnel ouvert
/// ici aurait donne une adresse que le QR n'annoncait pas. L'ouverture se fait
/// donc par le panneau des reglages, qui pilote le service. Reste ce que le
/// service ne dit pas : quels relais sont installes sur cette machine, et
/// comment obtenir celui qui manque.
fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "list_providers",
                "description": "Les relais de tunnel connus, et lesquels sont installes ici.                                 `needs_account` dit lequel exige une inscription avant de servir ;                                 `install_hint` dit comment obtenir celui qui manque.",
                "inputSchema": { "type": "object", "properties": {} }
            }
        ]
    })
}

async fn call_tool(name: &str, _args: Value) -> Result<Value, String> {
    match name {
        "list_providers" => Ok(json!({ "providers": list_providers() })),
        "start_remote_tunnel" | "stop_remote_tunnel" | "tunnel_status" => Err(
            "Le tunnel appartient au service local, pas a ce morph : ouvrez-le depuis              Reglages -> Serveur & fonctions, segment Tunnel."
                .to_string(),
        ),
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
