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

use serde::Serialize;

pub use tunnel::Provider;

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

// Ouvrir, interroger et fermer un tunnel appartenaient a ce morph. Ils n'y
// sont plus.
//
// Le service local en tenait deja un, par le crate `locaryn-travel`, et c'est
// le sien que lit le code d'appairage. Un second tunnel ouvert ici aurait donc
// donne une adresse que le QR ne portait pas : deux verites pour la meme
// question. Le panneau du morph pilote desormais celui du service, et ce qui
// reste ici est ce que le service ne dit pas — quels relais existent sur cette
// machine, avant qu'on en choisisse un.

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
}
