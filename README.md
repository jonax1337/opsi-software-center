<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/branding/soft-stack/banners/readme-banner-dark.png" />
  <img src="docs/branding/soft-stack/banners/readme-banner.png" alt="Software Center für opsi — Alles bereit für deine Arbeit." width="1600" />
</picture>

# Software Center für opsi

Ein Self-Service-Portal im Stil des **Microsoft Intune Company Portal** für Umgebungen, die
Software mit [opsi](https://www.opsi.org) verteilen. Anwender installieren, aktualisieren und
entfernen freigegebene Software selbst — ohne Admin-Rechte und ohne Ticket. Das Software Center
ersetzt die Oberfläche des `opsi-client-kiosk` und nutzt dieselbe Software-on-Demand-Mechanik.

- Startseite, App-Katalog (Kacheln oder Liste), Detailseite pro App, „Downloads und Updates",
  „Geräte" (Computerdetails, Selbsthilfe), „Hilfe und Support" mit Ihren IT-Kontaktdaten
- läuft im Infobereich, startet bei der Anmeldung und meldet neue Updates als
  Windows-Benachrichtigung
- passt sich an Windows 11 an (Mica, Akzentfarbe, Hell-/Dunkel-Modus)
- Tauri 2 + Svelte 5, kleiner nativer Installer

Logo, App-Icons und Banner: [Soft-Stack-Branding](docs/branding/soft-stack/README.md).

---

## Inhalt

1. [Voraussetzungen](#voraussetzungen)
2. [Installation in fünf Schritten](#installation-in-fünf-schritten)
3. [Produkt-Properties (Firmenname & Support-Kontakt)](#produkt-properties)
4. [Produkt-Icons](#produkt-icons)
5. [App-Katalog (Herausgeber, Kategorien, ausgewählte Apps)](#app-katalog)
6. [Den alten opsi-client-kiosk ablösen](#den-alten-opsi-client-kiosk-ablösen)
7. [Was Anwender sehen](#was-anwender-sehen)
8. [Fehlerbehebung](#fehlerbehebung)
8. [Wie es funktioniert & Sicherheit](#wie-es-funktioniert--sicherheit)
9. [Selbst bauen & entwickeln](#selbst-bauen--entwickeln)

---

## Voraussetzungen

| Wo | Was |
|---|---|
| opsi-Server | opsi 4.3 (`opsi-makepackage`, `opsi-package-manager`) |
| Clients | Windows 10/11 x64 mit **opsi-client-agent 4.3** |
| Clients | **Kiosk-API aktiv** — Standard; `opsiclientd.conf` → `[control_server]` → `kiosk_api_active = true` |
| Clients | **WebView2-Runtime** — bei Windows 11 vorinstalliert; unter Windows 10 lädt der Installer sie bei Bedarf nach (Internetzugang nötig) |
| Konfiguration | **Software on Demand** eingerichtet (siehe Schritt 3) |

Das Software Center braucht **keine Zugangsdaten**: Die Kiosk-API des lokalen opsiclientd ist
von `localhost` ohne Anmeldung erreichbar und zeigt nur die per Software on Demand
freigegebenen Produkte.

## Installation in fünf Schritten

### 1. Paket herunterladen

Laden Sie aus dem neuesten [Release](../../releases) die Datei
`software-center_<version>-opsi-source.tar.gz` herunter. Sie enthält die fertige
opsi-Paketquelle samt Installer. (Selbst bauen: siehe [unten](#selbst-bauen--entwickeln).)

### 2. Paket auf dem opsi-Server bauen und installieren

```bash
mkdir -p /var/lib/opsi/workbench/software-center
tar -xzf software-center_<version>-opsi-source.tar.gz -C /var/lib/opsi/workbench/software-center
cd /var/lib/opsi/workbench/software-center
opsi-makepackage
opsi-package-manager -i software-center_<version>-1.opsi
```

### 3. Software on Demand einrichten

Falls Sie den opsi-client-kiosk schon nutzen, ist das bereits erledigt — weiter mit Schritt 4.
Sonst in **opsi-configed**:

1. Unter *Produktkonfiguration* eine **Produktgruppe** anlegen (z. B. `self-service`) und die
   Produkte hineinlegen, die Anwender selbst installieren dürfen.
2. Unter *Server-Konfiguration → Host-Parameter* (global oder pro Client/Gruppe) setzen:
   - `software-on-demand.active` = `true`
   - `software-on-demand.product-group-ids` = `["self-service"]`

Details: [opsi-Handbuch, Kapitel opsi-client-kiosk / Software on Demand](https://docs.opsi.org).

### 4. Firmenname und Support-Kontakt setzen (optional, empfohlen)

In opsi-configed beim Produkt `software-center` die [Produkt-Properties](#produkt-properties)
ausfüllen — global oder pro Client-Gruppe. Ohne Angaben zeigt das Software Center neutrale Texte.

### 5. Auf die Clients ausrollen

Produkt `software-center` auf den gewünschten Clients auf **setup** stellen. Das Setup läuft
still im SYSTEM-Kontext, legt einen Startmenü-Eintrag an (keinen Desktop-Link), und das
Software Center richtet beim ersten Start den Autostart bei der Anmeldung ein. Erneute Läufe
sind schnell: Ist die Version schon installiert, werden nur Icons und Properties aktualisiert.

## Produkt-Properties

| Property | Wirkung | Beispiel |
|---|---|---|
| `company_name` | Überschrift der Startseite und Absender unter „Hilfe und Support" | `Beispiel GmbH` |
| `support_phone` | Telefon-Kachel unter „Hilfe und Support" (öffnet die Telefon-App) | `+49 30 1234567` |
| `support_email` | E-Mail-Kachel (öffnet das Mailprogramm) | `it@example.org` |
| `support_url` | Website-Kachel, muss mit `https://` oder `http://` beginnen | `https://helpdesk.example.org` |

Das Setup schreibt die Werte nach `HKLM\SOFTWARE\opsi-kiosk\Branding` (für Standardbenutzer nicht änderbar). Leere Properties
werden nicht angezeigt. Nach einer Änderung das Produkt erneut auf **setup** stellen.

## Produkt-Icons

Die opsi-Kiosk-API liefert keine Icons. Das Software Center sucht `<productId>.png`
(auch `.ico`, `.jpg`, `.svg`, `.webp`) in dieser Reihenfolge:

1. Umgebungsvariable `OPSI_KIOSK_ICON_DIR`
2. `%ProgramData%\opsi-kiosk\icons\` ← **wird vom opsi-Paket befüllt**
3. `icons\` neben der EXE
4. eine lokale opsi-client-kiosk-Installation (`ock_custom\product_icons`)
5. der Depot-Share `\\<depot>\opsi_depot\opsi-client-kiosk\files\app\ock_custom\product_icons\`

Dateiname = Produkt-ID, eine `IconsList.txt` wie beim alten Kiosk ist nicht nötig. Ohne Icon
erscheint ein neutrales Paketsymbol. Icons ins Paket legen Sie, indem Sie sie vor
`opsi-makepackage` nach `CLIENT_DATA/icons/` kopieren — oder beim Selbstbauen
`-IconSource <ordner>` angeben.

## App-Katalog

Herausgeber, Kategorien und „Ausgewählte Apps“ kennt die opsi-Kiosk-API nicht. Sie können sie
optional in einer `catalog.json` pflegen. Legen Sie die Datei vor `opsi-makepackage` nach
`CLIENT_DATA/catalog.json`; das Paket kopiert sie nach `%ProgramData%\opsi-kiosk\` (nur für
Administratoren beschreibbar). Alternativ liest das Software Center eine `catalog.json` neben der EXE.

```json
{
  "7zip":        { "publisher": "Igor Pavlov", "category": "Werkzeuge", "featured": true },
  "firefox-esr": { "publisher": "Mozilla",     "category": "Internet" },
  "vlc":         { "publisher": "VideoLAN" }
}
```

Schlüssel = Produkt-ID, jedes Feld ist optional. Damit erscheinen der Herausgeber auf Kacheln,
in Tabellen und auf der Detailseite, die Seite **App-Kategorien**, der Kategorie-Filter und
die Sortierung nach Herausgeber auf der Apps-Seite sowie das Banner **Ausgewählte Apps** auf der
Startseite. Ohne Datei fehlen nur diese Angaben; eine fehlerhafte Datei meldet das Software Center
einmalig mit Dateiname und Fehlerstelle.

## Den alten opsi-client-kiosk ablösen

1. `software-center` ausrollen (siehe oben).
2. `opsi-client-kiosk` auf den Clients auf **uninstall** stellen, z. B. für alle Clients, auf
   denen er installiert ist:
   ```bash
   opsi-admin -d method productOnClient_getObjects '[]' '{"productId":"opsi-client-kiosk","installationStatus":"installed"}' \
     | jq -r '.[].clientId' \
     | while read c; do opsi-admin -d method setProductActionRequest "opsi-client-kiosk" "$c" "uninstall"; done
   ```
3. Die Software-on-Demand-Konfiguration (`software-on-demand.*`, Event `on_demand`) **nicht**
   entfernen — das Software Center nutzt sie weiter.

## Was Anwender sehen

| Bereich | Inhalt |
|---|---|
| **Titelleiste** | in der Windows-Akzentfarbe: Zurück, Firmenname, App-Suche (Name oder Herausgeber), Konto-Avatar |
| **Startseite** | App-Karten (Updates, verfügbare und installierte Apps), Benachrichtigungen, Karte „Ihre Geräte“, Banner „Ausgewählte Apps“ |
| **Apps** | gesamter Katalog als Karten oder Tabelle; „Anzeigen“ (Alle / Installiert / Nicht installiert / Updates), „Kategorie“, „Sortieren nach“ (Name, Herausgeber) |
| **App-Kategorien** | eine Karte je Kategorie aus der `catalog.json` (nur sichtbar, wenn Kategorien gepflegt sind) |
| **App-Detailseite** | Herausgeber, Beschreibung, Versionen, Kategorie, Abhängigkeiten; *Installieren*, *Aktualisieren*, *Deinstallieren* starten sofort |
| **Downloads und Updates** | Tabellen „In der Warteschlange“ (mit Fortschrittsbalken), „Updates verfügbar“, „Installierte Apps“ (Menü „…“: Details, Neu installieren, Deinstallieren) |
| **Geräte** | Computerdetails, Wartungsinfos, Selbsthilfe (Richtlinien aktualisieren, DNS-Cache leeren, Richtlinienbericht) |
| **Hilfe und Support** | IT-Kontakt, Kontoinformationen |
| **Einstellungen** | Gerät synchronisieren, Autostart ein/aus, Verbindungsstatus, Version |

Das Fenster schließt in den Infobereich; beendet wird über das Tray-Menü. Alle 5 Minuten
prüft das Software Center im Hintergrund auf neue Zuweisungen und Updates und meldet sie per
Windows-Benachrichtigung.

## Fehlerbehebung

| Symptom | Ursache / Lösung |
|---|---|
| „Keine Verbindung zum opsi-Client-Dienst" | Läuft der Dienst `opsiclientd`? Ist `kiosk_api_active = true`? Antwortet `https://localhost:4441/kiosk`? Kurz nach der Anmeldung startet der Dienst manchmal noch — das Software Center versucht es automatisch erneut. |
| Keine Apps sichtbar | Dem Client ist keine Software-on-Demand-Gruppe zugewiesen, oder die Gruppe ist leer (Schritt 3). |
| Firmenname/Kontakt fehlt | Properties gesetzt? Danach `software-center` erneut auf **setup** stellen. Prüfen: Registry `HKLM\SOFTWARE\opsi-kiosk\Branding`. |
| Icons fehlen | Dateiname muss exakt der Produkt-ID entsprechen, siehe [Produkt-Icons](#produkt-icons). |
| Installation einer App schlägt fehl | Das Software Center setzt nur den Action-Request; die Installation führt opsi aus. Log auf dem Client: `C:\opsi.org\log\opsi-script.log`. |
| Paket-Setup schlägt fehl | Log des Setups: `C:\opsi.org\log\opsi-script.log`; der NSIS-Installer meldet seinen ExitCode dort. |

## Wie es funktioniert & Sicherheit

Die App spricht ausschließlich die **lokale Kiosk-API des opsiclientd** an
(`https://localhost:4441/kiosk`, JSON-RPC) — dieselbe Schnittstelle wie der offizielle Kiosk:

| Methode | Zweck |
|---|---|
| `getClientId` | opsi-Client-ID dieses Rechners |
| `getConfigDataFromOpsiclientd` | Depot-ID, Agent-Version |
| `getKioskProductInfosForClient` | Produktkatalog + Installationsstatus |
| `setProductActionRequestWithDependencies` | setup/uninstall setzen (inkl. Abhängigkeiten) |
| `processActionRequests` | Action-Requests sofort ausführen |

- Sichtbar und auslösbar ist nur, was serverseitig per Software on Demand freigegeben ist.
- Die TLS-Prüfung ist nur für `localhost:4441` deaktiviert (selbstsigniertes
  opsiclientd-Zertifikat). Entfernte Hosts werden nur lesend für Icons angesprochen (Depot-Share).
- Produkt-IDs werden vor jeder Pfadverwendung gegen Traversal geprüft; Support-Links öffnet die
  App nur aus der Admin-Konfiguration und nur als `mailto:`, `tel:` oder `http(s)://`.
- Ein Demo-Katalog erscheint nur im Entwicklungs-Build oder mit `OPSI_KIOSK_MOCK=1` — nie
  versehentlich bei Anwendern.

## Selbst bauen & entwickeln

Voraussetzungen: Windows, Node.js ≥ 20, pnpm, Rust (stable) mit MSVC-Build-Tools.

```powershell
pnpm install
pnpm tauri:dev        # App mit Hot-Reload; ohne opsiclientd mit Demo-Katalog
pnpm check            # svelte-check (muss 0 Fehler melden)
pnpm test             # Frontend-Tests (vitest)
cd src-tauri; cargo test; cd ..   # Rust-Tests

.\deploy\build-opsi-package.ps1   # Release bauen + opsi-Paketquelle nach dist-opsi\
.\deploy\build-opsi-package.ps1 -SkipBuild -IconSource <ordner-mit-icons>
```

**Neue Version veröffentlichen:** Version in `package.json` und `src-tauri/Cargo.toml` erhöhen
(`tauri.conf.json` liest sie aus `package.json`; `control` und `setup.opsiscript` setzt das
Build-Skript), committen und ein Tag `v<version>` pushen. Der GitHub-Workflow
[`build.yml`](.github/workflows/build.yml) baut Installer und opsi-Paketquelle und hängt sie an
das Release.

Architektur und Konventionen: [`CLAUDE.md`](CLAUDE.md) · Paket-Interna:
[`deploy/opsi/README.md`](deploy/opsi/README.md)
