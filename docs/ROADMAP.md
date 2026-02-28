# AIUX - Roadmap

> Fokus: Raspi einrichten und erstes LLM-Gespräch auf dem System.

---

## Phase 1: Basis-System auf dem Raspi

### 1.1 Alpine Linux auf SD-Karte

- [ ] Alpine aarch64 Image herunterladen
- [ ] Image auf SD-Karte flashen (dd oder balenaEtcher)
- [ ] Erster Boot am HDMI-Monitor
- [ ] Grundkonfiguration (alpine-setup: Tastatur, Timezone, Hostname)

### 1.2 Netzwerk

- [ ] Ethernet oder WiFi einrichten
- [ ] SSH aktivieren (für Remote-Zugriff vom Hauptrechner)
- [ ] Testen: Ping nach aussen, DNS funktioniert

### 1.3 System härten (Minimum)

- [ ] Root-Passwort setzen
- [ ] Eigenen User anlegen (bruce)
- [ ] LLM-User anlegen (aiux)
- [ ] SSH nur mit Key-Auth
- [ ] Basis-Firewall (iptables/nftables)

### 1.4 Erster LLM-Call

- [ ] curl + jq installieren (apk add curl jq)
- [ ] API-Key sicher ablegen (~aiux/.env oder secrets)
- [ ] Erster manueller API-Call (curl an Anthropic)
- [ ] Einfaches Shell-Skript: Eingabe → API → Antwort

### 1.5 Agent-Loop (Shell-Version)

- [ ] Interaktive Schleife: Prompt → LLM → Anzeige
- [ ] LLM kann Shell-Befehle vorschlagen
- [ ] Mensch bestätigt Befehle vor Ausführung
- [ ] Ergebnis wird ans LLM zurückgegeben

---

## Phase 2+ (Ausblick, noch nicht geplant)

- aiux-agent in Go umschreiben
- aiux-hub Architektur
- Erster aiux-nerve (log-brain)
- MCP-Integration
- TUI-Interface
- Touch-Display einrichten
- Remote-Zugang

---

*Letzte Aktualisierung: 2026-02-28*
