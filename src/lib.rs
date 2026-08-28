//! Rendre cette machine joignable de l'extérieur, sans toucher à la box.
//!
//! Le tunnel est **sortant** : l'ordinateur appelle un relais, et c'est au
//! relais que le téléphone parle. Rien n'a besoin d'être ouvert sur la box, ce
//! qui est tout l'intérêt — la personne visée est dans un hôtel, pas devant son
//! routeur.
//!
//! Trois relais, parce qu'aucun ne convient à tout le monde : Cloudflare ne
//! demande aucun compte, ngrok est celui que beaucoup ont déjà, et les tunnels
//! Microsoft sont ceux qu'un employeur autorise le plus volontiers. Aucun n'est
//! livré avec le morph : embarquer le binaire d'un tiers, c'est embarquer ses
//! mises à jour et ses failles. On le détecte, et s'il manque on dit comment
//! l'obtenir.
//!
//! Le tunnel ouvert vit dans le processus du serveur MCP. Un appel l'ouvre, un
//! autre l'interroge, un troisième le ferme — il ne se referme pas tout seul
//! entre deux messages.

pub mod tunnel;

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;

pub use tunnel::{Provider, Tunnel, TunnelError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRequest {
    /// `cloudflare`, `ngrok` ou `devtunnel`. Vide : le premier relais présent
    /// sur cette machine, Cloudflare en tête puisqu'il ne demande pas de compte.
    #[serde(default)]
    pub provider: String,
    /// Le port que sert le service local.
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Ce qu'on peut dire d'un relais avant que la personne ne choisisse.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub binary: &'static str,
    /// Présent sur cette machine.
    pub available: bool,
    /// Exige un compte ou une connexion préalable. Dit avant le choix, plutôt
    /// que découvert comme un échec après.
    pub needs_account: bool,
    pub install_hint: &'static str,
}

/// Le tunnel en cours, s'il y en a un.
///
/// Un seul à la fois : deux relais pointant le même port ne rendent pas la
/// machine plus joignable, ils rendent seulement l'adresse ambiguë.
struct Running {
    tunnel: Tunnel,
    port: u16,
}

fn slot() -> &'static Mutex<Option<Running>> {
    static SLOT: OnceLock<Mutex<Option<Running>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Les relais connus, et lesquels sont utilisables ici.
pub fn list_providers() -> Vec<ProviderInfo> {
    Provider::ALL
        .into_iter()
        .map(|p| ProviderInfo {
            id: p.id(),
            label: p.label(),
            binary: p.binary(),
            available: p.is_available(),
            needs_account: p.needs_account(),
            install_hint: p.install_hint(),
        })
        .collect()
}

/// Le relais à employer quand personne n'en a nommé un.
///
/// Cloudflare d'abord : c'est le seul qui n'exige ni compte ni connexion, donc
/// le seul qui a des chances d'aboutir sans préparation.
fn default_provider() -> Result<Provider, String> {
    Provider::ALL
        .into_iter()
        .find(|p| p.is_available())
        .ok_or_else(|| {
            let aides: Vec<String> = Provider::ALL
                .into_iter()
                .map(|p| format!("- {} : {}", p.label(), p.install_hint()))
                .collect();
            format!(
                "Aucun relais n'est installé sur cette machine. Au choix :\n{}",
                aides.join("\n")
            )
        })
}

/// Ouvrir un tunnel vers `port` et attendre que le relais annonce l'adresse.
pub async fn start_remote_tunnel(req: TunnelRequest) -> Result<TunnelStatus, String> {
    if req.port == 0 {
        return Err("Le port à exposer doit être précisé.".into());
    }

    let provider = match req.provider.trim() {
        "" => default_provider()?,
        nom => Provider::parse(nom).ok_or_else(|| {
            format!(
                "Relais inconnu : « {nom} ». Ceux que je connais : {}.",
                Provider::ALL
                    .into_iter()
                    .map(|p| p.id())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?,
    };

    let mut garde = slot().lock().await;
    if let Some(en_cours) = garde.as_ref() {
        return Err(format!(
            "Un tunnel est déjà ouvert vers le port {} ({}), à l'adresse {}. \
             Fermez-le avant d'en ouvrir un autre.",
            en_cours.port,
            en_cours.tunnel.provider.label(),
            en_cours.tunnel.url
        ));
    }

    let tunnel = tunnel::start(provider, req.port)
        .await
        .map_err(|e| e.to_string())?;
    let etat = TunnelStatus {
        active: true,
        public_url: Some(tunnel.url.clone()),
        provider: Some(provider.id().to_string()),
        port: Some(req.port),
    };
    *garde = Some(Running {
        tunnel,
        port: req.port,
    });
    Ok(etat)
}

/// L'état du tunnel, sans rien ouvrir ni fermer.
pub async fn tunnel_status() -> TunnelStatus {
    match slot().lock().await.as_ref() {
        Some(r) => TunnelStatus {
            active: true,
            public_url: Some(r.tunnel.url.clone()),
            provider: Some(r.tunnel.provider.id().to_string()),
            port: Some(r.port),
        },
        None => TunnelStatus {
            active: false,
            public_url: None,
            provider: None,
            port: None,
        },
    }
}

/// Fermer le tunnel. Fermer quand il n'y en a pas n'est pas une erreur : le
/// résultat voulu est déjà obtenu.
pub async fn stop_remote_tunnel() -> TunnelStatus {
    if let Some(r) = slot().lock().await.take() {
        r.tunnel.stop().await;
    }
    TunnelStatus {
        active: false,
        public_url: None,
        provider: None,
        port: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_trois_relais_sont_decrits() {
        let v = list_providers();
        assert_eq!(v.len(), 3);
        let cf = v.iter().find(|p| p.id == "cloudflare").expect("cloudflare");
        assert!(
            !cf.needs_account,
            "Cloudflare est le seul sans compte : c'est ce qui en fait le défaut"
        );
        assert!(v.iter().filter(|p| p.needs_account).count() == 2);
        assert!(v.iter().all(|p| !p.install_hint.is_empty()));
    }

    #[test]
    fn un_relais_inconnu_est_nomme_avec_la_liste() {
        let err = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(start_remote_tunnel(TunnelRequest {
                provider: "monrelais".into(),
                port: 7443,
            }))
            .unwrap_err();
        assert!(err.contains("monrelais"), "{err}");
        assert!(err.contains("cloudflare"), "{err}");
    }

    #[test]
    fn un_port_absent_est_refuse_sans_lancer_de_processus() {
        let err = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(start_remote_tunnel(TunnelRequest {
                provider: "cloudflare".into(),
                port: 0,
            }))
            .unwrap_err();
        assert!(err.contains("port"), "{err}");
    }

    /// Sans tunnel, l'état doit le dire — et non pas manquer.
    #[test]
    fn l_etat_au_repos_est_inactif() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let e = rt.block_on(tunnel_status());
        assert!(!e.active);
        assert!(e.public_url.is_none());
        // Fermer sans tunnel ouvert n'est pas une erreur.
        let f = rt.block_on(stop_remote_tunnel());
        assert!(!f.active);
    }
}
