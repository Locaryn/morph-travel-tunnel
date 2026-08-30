//! Quels relais de tunnel sont installés sur cette machine.
//!
//! Le tunnel lui-même appartient au service local : il doit survivre à la
//! fermeture de la fenêtre, ce qui est toute la situation visée. Ce module ne
//! répond qu'à la question posée *avant* d'en ouvrir un.
//!
//! Trois relais, parce qu'aucun ne convient à tout le monde : Cloudflare ne
//! demande aucun compte, ngrok est celui que beaucoup ont déjà, et les tunnels
//! Microsoft sont ceux qu'un employeur autorise le plus volontiers.
//!
//! Aucun n'est livré avec le morph. Embarquer le binaire d'un tiers, c'est
//! embarquer ses mises à jour et ses failles ; on le détecte, et s'il manque on
//! dit exactement comment l'obtenir.

// ── Trouver l'exécutable ────────────────────────────────────────────────────
//
// Inliné plutôt qu'importé du socle : ce morph doit se compiler seul. Sous
// Windows un outil installé par winget s'appelle souvent `x.cmd` ou `x.exe`,
// et `CreateProcess` ne devine pas l'extension — un `cloudflared` bien présent
// passerait alors pour absent.

fn resolve_program(command: &str) -> std::ffi::OsString {
    #[cfg(not(windows))]
    {
        std::ffi::OsString::from(command)
    }
    #[cfg(windows)]
    {
        const EXTS: [&str; 4] = [".exe", ".cmd", ".bat", ".com"];
        let p = std::path::Path::new(command);
        if command.contains('/') || command.contains('\\') {
            if p.is_file() {
                return std::ffi::OsString::from(command);
            }
            for ext in EXTS {
                let c = std::path::PathBuf::from(format!("{command}{ext}"));
                if c.is_file() {
                    return c.into_os_string();
                }
            }
            return std::ffi::OsString::from(command);
        }
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                for ext in EXTS {
                    let c = dir.join(format!("{command}{ext}"));
                    if c.is_file() {
                        return c.into_os_string();
                    }
                }
            }
        }
        std::ffi::OsString::from(command)
    }
}

fn program_exists(command: &str) -> bool {
    let resolved = resolve_program(command);
    if std::path::Path::new(&resolved).is_file() {
        return true;
    }
    // Sous Unix le résolveur rend le nom tel quel : c'est ici qu'on cherche.
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|d| d.join(command).is_file()))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Cloudflare,
    Ngrok,
    DevTunnel,
}

impl Provider {
    pub const ALL: [Provider; 3] = [Provider::Cloudflare, Provider::Ngrok, Provider::DevTunnel];

    pub fn id(&self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare",
            Self::Ngrok => "ngrok",
            Self::DevTunnel => "devtunnel",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|p| p.id() == s.trim().to_ascii_lowercase())
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Cloudflare => "Cloudflare",
            Self::Ngrok => "ngrok",
            Self::DevTunnel => "Tunnels Microsoft",
        }
    }

    /// The executable to look for.
    pub fn binary(&self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflared",
            Self::Ngrok => "ngrok",
            Self::DevTunnel => "devtunnel",
        }
    }

    /// What the user has to do to get it, in one sentence.
    pub fn install_hint(&self) -> &'static str {
        match self {
            Self::Cloudflare => {
                "Installez cloudflared (winget install Cloudflare.cloudflared, \
                 brew install cloudflared, ou le paquet .deb de Cloudflare). Aucun compte requis."
            }
            Self::Ngrok => {
                "Installez ngrok depuis ngrok.com, puis exécutez une fois \
                 « ngrok config add-authtoken <votre jeton> »."
            }
            Self::DevTunnel => {
                "Installez devtunnel (winget install Microsoft.devtunnel), puis \
                 exécutez une fois « devtunnel user login »."
            }
        }
    }

    /// Whether an account or a prior login is needed. Shown before the user
    /// picks, rather than discovered as a failure afterwards.
    pub fn needs_account(&self) -> bool {
        !matches!(self, Self::Cloudflare)
    }

    pub fn is_available(&self) -> bool {
        program_exists(self.binary())
    }
}

// Lancer le relais, lire son adresse et arreter le processus vivaient ici.
// Ils vivent maintenant dans le service local, qui tenait deja la meme chose :
// deux tunnels ouverts pour la meme machine donnent deux adresses, et le code
// d'appairage n'en porte qu'une. Ce module ne fait donc plus que ce que le
// service ne fait pas — dire lesquels de ces outils sont sur cette machine.

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_missing_tool_is_named_along_with_the_way_to_get_it() {
        // Every provider must answer "what do I install?" before anything is
        // spawned — the alternative is a raw "program not found".
        for p in Provider::ALL {
            let hint = p.install_hint();
            assert!(hint.len() > 40, "consigne trop vague pour {}", p.id());
            assert!(
                hint.contains("Installez") || hint.contains("installez"),
                "consigne sans action pour {}",
                p.id()
            );
        }
        // And the round trip through the identifier used in configuration.
        for p in Provider::ALL {
            assert_eq!(Provider::parse(p.id()), Some(p));
        }
        assert_eq!(Provider::parse("inconnu"), None);
    }
}
