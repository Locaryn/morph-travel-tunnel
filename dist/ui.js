/**
 * Les panneaux d'appairage qu'ajoute ce morph.
 *
 * L'application ne connait que le reseau local. Les deux transports que ce
 * morph apporte — un port joignable de l'exterieur, et un tunnel sortant — se
 * declarent comme segments du meme selecteur, et se dessinent ici.
 *
 * Light DOM et classes de l'hote, jamais de shadow DOM : un panneau qui
 * emporte ses propres couleurs se voit tout de suite, et cesse de suivre le
 * theme des que l'application change le sien. Aucune valeur n'est inventee —
 * quand le service ne repond pas, on le dit.
 */
(function () {
  "use strict";

  /** Ce que l'hote a prete au panneau, ou rien s'il est monte hors contexte. */
  function hote(el) {
    const c = el.context;
    return c && typeof c.demanderCode === "function" ? c : null;
  }

  // Une preference d'affichage, gardee sur cette machine seulement. Enveloppee
  // parce qu'un navigateur en navigation privee refuse d'ecrire, et qu'un
  // panneau ne doit pas se briser pour si peu.
  function lire(cle, defaut) {
    try {
      const v = window.localStorage.getItem(cle);
      return v === null ? defaut : v;
    } catch {
      return defaut;
    }
  }

  function ecrire(cle, valeur) {
    try {
      window.localStorage.setItem(cle, valeur);
    } catch {
      // Stockage indisponible : la valeur vaut pour cette session, c'est tout.
    }
  }

  function vider(el) {
    while (el.firstChild) el.removeChild(el.firstChild);
  }

  function creer(tag, classe, texte) {
    const el = document.createElement(tag);
    if (classe) el.className = classe;
    if (texte !== undefined) el.textContent = texte;
    return el;
  }

  /** Une note explicative, avec le style des notices de l'hote. */
  function notice(texte) {
    const d = creer("div", "locaryn-pairing-notice");
    d.appendChild(creer("span", null, texte));
    return d;
  }

  function avertissement(texte) {
    return creer("div", "locaryn-pairing-warning", texte);
  }

  /**
   * Le bloc du QR, identique a celui de l'hote.
   *
   * Le SVG vient du service local par la fonction que l'hote a pretee : il
   * n'est pas fabrique ici, et rien ne s'affiche tant qu'il n'est pas arrive.
   */
  function blocQr(code, surCopie) {
    const bloc = creer("div", "locaryn-travel-code");

    const bouton = creer("button", "locaryn-travel-qr");
    bouton.type = "button";
    bouton.title = "Code d'appairage";
    // Le SVG est produit par le service local de la machine, pas par le reseau.
    bouton.innerHTML = code.qr_svg;
    bloc.appendChild(bouton);

    const dire = creer("div", "locaryn-travel-say");
    dire.appendChild(creer("p", "locaryn-travel-title", "Scannez avec le téléphone"));
    dire.appendChild(
      creer(
        "p",
        "locaryn-travel-sub",
        "Le code porte l'adresse et l'empreinte de cette machine. Rien à saisir.",
      ),
    );

    const actions = creer("div", "locaryn-pairing-actions");
    const copier = creer("button", "locaryn-btn-ghost", "Copier l'adresse");
    copier.type = "button";
    copier.addEventListener("click", () => {
      navigator.clipboard
        .writeText(code.url)
        .then(() => {
          copier.textContent = "Adresse copiée";
          window.setTimeout(() => {
            copier.textContent = "Copier l'adresse";
          }, 1500);
        })
        .catch((e) => surCopie(String(e)));
    });
    actions.appendChild(copier);
    dire.appendChild(actions);

    bloc.appendChild(dire);
    return bloc;
  }

  // ── Accès distant : une adresse que l'utilisateur connaît ──────────────
  //
  // Le morph ne devine pas l'adresse publique : personne d'autre que la
  // personne qui a redirigé son port ne la connaît. On la demande, et le
  // service local dessine le code qui la porte.

  class PanneauPublic extends HTMLElement {
    constructor() {
      super();
      this.adresse = "";
      this.code = null;
      this.erreur = null;
      this.occupe = false;
    }

    connectedCallback() {
      this.rendre();
    }

    async generer() {
      const h = hote(this);
      if (!h) {
        this.erreur = "Ce panneau n'a pas reçu le contexte de l'application.";
        this.rendre();
        return;
      }
      this.occupe = true;
      this.erreur = null;
      this.code = null;
      this.rendre();
      try {
        this.code = await h.demanderCode("public", this.adresse.trim());
      } catch (e) {
        this.code = null;
        this.erreur = String(e && e.message ? e.message : e);
      } finally {
        this.occupe = false;
        this.rendre();
      }
    }

    rendre() {
      const h = hote(this);
      vider(this);

      if (h && !h.serveurActif) {
        this.appendChild(
          notice(
            "Activez le partage réseau dans la colonne de gauche : sans service en écoute, une adresse publique ne mène nulle part.",
          ),
        );
        return;
      }

      const bloc = creer("div", "locaryn-pairing-public");
      const etiquette = creer(
        "label",
        "locaryn-pairing-select-label",
        "Adresse publique et port",
      );
      etiquette.htmlFor = "morph-remote-adresse";
      bloc.appendChild(etiquette);

      const ligne = creer("div", "locaryn-pairing-public-row");
      const champ = creer("input", "locaryn-input");
      champ.id = "morph-remote-adresse";
      champ.placeholder = "maison.exemple:7443";
      champ.value = this.adresse;
      champ.addEventListener("input", (e) => {
        this.adresse = e.target.value;
        const b = this.querySelector("[data-role=generer]");
        if (b) b.disabled = this.occupe || this.adresse.trim() === "";
      });
      champ.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && this.adresse.trim() !== "") void this.generer();
      });
      ligne.appendChild(champ);

      const bouton = creer("button", "locaryn-btn-ghost", this.occupe ? "Génération…" : "Générer");
      bouton.type = "button";
      bouton.setAttribute("data-role", "generer");
      bouton.disabled = this.occupe || this.adresse.trim() === "";
      bouton.addEventListener("click", () => void this.generer());
      ligne.appendChild(bouton);

      bloc.appendChild(ligne);
      this.appendChild(bloc);

      if (this.erreur) this.appendChild(avertissement(this.erreur));
      if (this.code && this.code.qr_svg) {
        this.appendChild(
          blocQr(this.code, (m) => {
            this.erreur = m;
            this.rendre();
          }),
        );
      }
    }
  }

  // ── Tunnel : un relais appelé depuis l'intérieur ───────────────────────
  //
  // Rien n'est ouvert sur la box : c'est la machine qui appelle le relais.
  // Aucun relais n'est livré avec le morph — on regarde lesquels sont là, et
  // on dit comment obtenir celui qui manque plutôt que d'échouer sans raison.

  class PanneauTunnel extends HTMLElement {
    constructor() {
      super();
      this.relais = [];
      this.choisi = "";
      this.etat = null;
      this.code = null;
      this.erreur = null;
      this.occupe = false;
      this.charge = false;
      // Le serveur vers lequel renvoyer, quand le relais en reclame un. Garde
      // d'une session a l'autre : personne ne veut retaper son serveur.
      this.cible = lire("morph-remote-cible", "");
    }

    connectedCallback() {
      this.rendre();
      void this.rafraichir();
    }

    async rafraichir() {
      const h = hote(this);
      if (!h || typeof h.relaisDisponibles !== "function") {
        this.erreur = "Ce panneau n'a pas reçu le contexte de l'application.";
        this.charge = true;
        this.rendre();
        return;
      }
      try {
        const [relais, etat] = await Promise.all([h.relaisDisponibles(), h.etatPartage()]);
        this.relais = relais || [];
        this.etat = etat || null;
        if (!this.choisi) {
          // Celui qui ne demande rien d'abord : découvrir qu'il faut un compte
          // au moment de partir est le pire moment pour l'apprendre.
          const simple = this.relais.find((r) => r.installed && !r.needs_account);
          this.choisi = (simple || this.relais.find((r) => r.installed) || this.relais[0] || {}).id || "";
        }
        this.erreur = null;
      } catch (err) {
        this.erreur = String(err && err.message ? err.message : err);
      } finally {
        this.charge = true;
        this.rendre();
        if (this.etat && this.etat.active) void this.produireCode();
      }
    }

    async produireCode() {
      const h = hote(this);
      if (!h) return;
      try {
        this.code = await h.demanderCode("tunnel");
        this.erreur = null;
      } catch (e) {
        this.code = null;
        this.erreur = String(e && e.message ? e.message : e);
      }
      this.rendre();
    }

    async basculer() {
      const h = hote(this);
      if (!h) return;
      this.occupe = true;
      this.erreur = null;
      this.rendre();
      try {
        const actif = Boolean(this.etat && this.etat.active);
        const relais = this.relais.find((r) => r.id === this.choisi);
        // Un seul champ pour le service : le relais, puis ce qu'il lui faut.
        const demande =
          relais && relais.needs_target ? `${this.choisi}:${this.cible.trim()}` : this.choisi;
        this.etat = await h.reglerPartage(actif ? null : demande);
        if (!this.etat || !this.etat.active) this.code = null;
        await this.rafraichir();
      } catch (err) {
        this.erreur = String(err && err.message ? err.message : err);
      } finally {
        this.occupe = false;
        this.rendre();
      }
    }

    rendre() {
      const h = hote(this);
      vider(this);

      if (h && !h.serveurActif) {
        this.appendChild(
          notice(
            "Activez le partage réseau dans la colonne de gauche : un tunnel qui ne mène à aucun service ouvert ne sert à rien.",
          ),
        );
        return;
      }

      if (!this.charge) {
        this.appendChild(creer("p", "locaryn-field-hint", "Lecture des relais disponibles…"));
        return;
      }

      const actif = Boolean(this.etat && this.etat.active);

      const bloc = creer("div", "locaryn-pairing-public");
      const etiquette = creer("label", "locaryn-pairing-select-label", "Relais");
      etiquette.htmlFor = "morph-remote-relais";
      bloc.appendChild(etiquette);

      const ligne = creer("div", "locaryn-pairing-public-row");
      const choix = creer("select", "locaryn-input");
      choix.id = "morph-remote-relais";
      choix.disabled = this.occupe || actif;
      for (const r of this.relais) {
        const o = document.createElement("option");
        o.value = r.id;
        // Ce qui manque est dit avant le choix, pas découvert comme un échec.
        o.textContent = r.installed
          ? r.needs_account
            ? `${r.label} — compte requis`
            : r.label
          : `${r.label} — non installé`;
        if (r.id === this.choisi) o.selected = true;
        choix.appendChild(o);
      }
      choix.addEventListener("change", (e) => {
        this.choisi = e.target.value;
        this.rendre();
      });
      ligne.appendChild(choix);

      const bouton = creer(
        "button",
        actif ? "locaryn-btn-ghost" : "locaryn-btn-primary",
        this.occupe ? "…" : actif ? "Fermer le tunnel" : "Ouvrir le tunnel",
      );
      bouton.type = "button";
      bouton.setAttribute("data-role", "basculer");
      const relaisChoisi = this.relais.find((r) => r.id === this.choisi);
      const cibleManquante =
        Boolean(relaisChoisi && relaisChoisi.needs_target) && this.cible.trim() === "";
      bouton.disabled = this.occupe || (!actif && (this.choisi === "" || cibleManquante));
      bouton.addEventListener("click", () => void this.basculer());
      ligne.appendChild(bouton);

      bloc.appendChild(ligne);
      this.appendChild(bloc);

      const relais = this.relais.find((r) => r.id === this.choisi);

      if (relais && relais.needs_target) {
        const bloc2 = creer("div", "locaryn-pairing-public");
        const et2 = creer("label", "locaryn-pairing-select-label", "Votre serveur");
        et2.htmlFor = "morph-remote-cible";
        bloc2.appendChild(et2);
        const champ = creer("input", "locaryn-input");
        champ.id = "morph-remote-cible";
        champ.placeholder = "moi@serveur.fr:8443";
        champ.value = this.cible;
        champ.disabled = this.occupe || actif;
        champ.addEventListener("input", (e) => {
          this.cible = e.target.value;
          ecrire("morph-remote-cible", this.cible);
          const b = this.querySelector("[data-role=basculer]");
          if (b) b.disabled = this.occupe || this.cible.trim() === "";
        });
        bloc2.appendChild(champ);
        bloc2.appendChild(
          creer(
            "p",
            "locaryn-field-hint",
            "Le port que le serveur ouvrira. Ajoutez « /2222 » si son SSH n'écoute pas sur 22. La clé vient de votre agent SSH — Locaryn n'en manipule aucune.",
          ),
        );
        this.appendChild(bloc2);
      }

      if (relais && !relais.installed && !actif) {
        this.appendChild(notice(relais.install_hint));
      }

      if (this.etat && this.etat.blocker) this.appendChild(notice(this.etat.blocker));
      if (this.erreur) this.appendChild(avertissement(this.erreur));

      if (actif && !this.code && !this.erreur) {
        this.appendChild(creer("p", "locaryn-field-hint", "Tunnel ouvert — préparation du code…"));
      }

      if (this.code && this.code.qr_svg) {
        this.appendChild(
          blocQr(this.code, (m) => {
            this.erreur = m;
            this.rendre();
          }),
        );
      }
    }
  }

  if (!customElements.get("locaryn-pairing-public")) {
    customElements.define("locaryn-pairing-public", PanneauPublic);
  }
  if (!customElements.get("locaryn-pairing-tunnel")) {
    customElements.define("locaryn-pairing-tunnel", PanneauTunnel);
  }
})();
