(function () {
  "use strict";

  const CSS = `
:host { display: block; width: 100%; color: var(--text, #e8edf5); font-family: inherit; box-sizing: border-box; }
* { box-sizing: border-box; }
.panel-container { width: 100%; max-width: 920px; margin: 0 auto; display: flex; flex-direction: column; gap: 16px; }
.header-card {
  display: flex; align-items: center; justify-content: space-between; padding: 16px 20px;
  background: var(--surface, rgba(255, 255, 255, 0.035)); border: 1px solid var(--border, rgba(255, 255, 255, 0.1));
  border-radius: var(--radius, 12px);
}
.title-wrap { display: flex; align-items: center; gap: 12px; }
.icon-box {
  width: 40px; height: 40px; border-radius: 10px; background: rgba(var(--accent-rgb, 110, 168, 254), 0.15);
  color: var(--accent, #6ea8fe); display: grid; place-items: center; font-size: 20px;
}
.title { font-size: 16px; font-weight: 700; color: var(--text, #e8edf5); }
.subtitle { font-size: 12px; color: var(--text-faint, #96a3b8); margin-top: 2px; }
.badge {
  display: inline-flex; align-items: center; padding: 4px 10px; border-radius: 99px; font-size: 11px;
  font-weight: 600; background: rgba(101, 211, 145, 0.12); color: #65d391; border: 1px solid rgba(101, 211, 145, 0.25);
}
.field-card {
  display: flex; flex-direction: column; gap: 10px; background: var(--surface, rgba(255, 255, 255, 0.035));
  border: 1px solid var(--border, rgba(255, 255, 255, 0.1)); border-radius: var(--radius, 12px); padding: 16px;
}
.label { font-size: 11px; font-weight: 700; color: var(--text-dim, #94a3b8); text-transform: uppercase; letter-spacing: 0.06em; }
.select {
  width: 100%; border: 1px solid var(--border, rgba(255, 255, 255, 0.14)); border-radius: var(--radius-sm, 8px);
  background: var(--bg, rgba(0, 0, 0, 0.25)); color: inherit; padding: 10px 12px; font: inherit; font-size: 13px; outline: none;
}
.btn-primary {
  width: 100%; padding: 12px; background: var(--accent, #6ea8fe); color: #0b101b; border: none;
  border-radius: var(--radius-sm, 8px); font-weight: 700; font-size: 14px; cursor: pointer;
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
`;

  class LocarynRemotePanel extends HTMLElement {
    constructor() {
      super();
      this.attachShadow({ mode: "open" });
      this.provider = "cloudflare";
      this.isTunneling = false;
      this.status = null;
    }
    connectedCallback() { this.render(); }

    async toggleTunnel() {
      this.isTunneling = true;
      this.render();
      try {
        const bridge = window.locaryn || window.LocarynPluginAPI;
        if (bridge && bridge.invokeExtensionTool) {
          const res = await bridge.invokeExtensionTool("start_remote_tunnel", {
            provider: this.provider,
            port: 54321
          });
          this.status = typeof res === "string" ? JSON.parse(res) : res;
        } else {
          this.status = { active: true, public_url: "https://locaryn-demo.trycloudflare.com" };
        }
      } catch (err) {
        alert("Erreur de tunnel: " + err);
      } finally {
        this.isTunneling = false;
        this.render();
      }
    }

    render() {
      this.shadowRoot.innerHTML = `
        <style>${CSS}</style>
        <div class="panel-container">
          <div class="header-card">
            <div class="title-wrap">
              <div class="icon-box">☁️</div>
              <div>
                <div class="title">Tunnels & Connexion Distante</div>
                <div class="subtitle">Accès sécurisé depuis votre smartphone en déplacement (Travel Mode)</div>
              </div>
            </div>
            <div class="badge">Actif</div>
          </div>

          <div class="field-card">
            <label class="label">Fournisseur de tunnel chiffré</label>
            <select class="select" id="tun-prov">
              <option value="cloudflare" ${this.provider === "cloudflare" ? "selected" : ""}>Cloudflare Tunnel (Recommandé - Sans compte)</option>
              <option value="ngrok" ${this.provider === "ngrok" ? "selected" : ""}>ngrok Secure Ingress</option>
              <option value="devtunnel" ${this.provider === "devtunnel" ? "selected" : ""}>Microsoft Dev Tunnel</option>
            </select>
          </div>

          <button class="btn-primary" id="tun-btn" ${this.isTunneling ? "disabled" : ""}>
            ${this.isTunneling ? "Activation du tunnel..." : "Démarrer le tunnel distant"}
          </button>

          ${this.status && this.status.public_url ? `
            <div class="field-card" style="margin-top: 10px;">
              <label class="label">URL Publique Sécurisée</label>
              <div style="font-size: 14px; font-weight: 700; color: #65d391; padding: 6px 0;">
                ${this.status.public_url}
              </div>
              <div style="font-size: 12px; color: var(--text-dim);">
                Scannez le QR code depuis l'application mobile Locaryn pour appairer votre téléphone.
              </div>
            </div>
          ` : ""}
        </div>
      `;

      const provEl = this.shadowRoot.querySelector("#tun-prov");
      if (provEl) {
        provEl.addEventListener("change", (e) => { this.provider = e.target.value; });
      }

      const btn = this.shadowRoot.querySelector("#tun-btn");
      if (btn) btn.addEventListener("click", () => this.toggleTunnel());
    }
  }

  if (!customElements.get("locaryn-remote-panel")) {
    customElements.define("locaryn-remote-panel", LocarynRemotePanel);
  }
})();
