# Software Center

Leichtgewichtiger, moderner Ersatz für den opsi-client-kiosk: Endanwender installieren,
aktualisieren und entfernen freigegebene Software selbst — ohne Admin-Rechte, ohne Ticket.

Tauri 2 + Svelte 5 + Tailwind v4. Windows 11 Mica-Look, Hell-/Dunkel-Modus folgt dem System.

![Stack](https://img.shields.io/badge/Tauri-2-blue) ![Svelte](https://img.shields.io/badge/Svelte-5-orange)

## Wie es funktioniert

Die App spricht die **lokale Kiosk-API des opsiclientd** an — dieselbe Schnittstelle, die der
offizielle opsi-client-kiosk verwendet:

```
https://localhost:4441/kiosk   (JSON-RPC, unauthentifiziert von localhost)
```

Verwendete Methoden:

| Methode | Zweck |
|---|---|
| `getClientId` | opsi-Client-ID dieses Rechners |
| `getConfigDataFromOpsiclientd` | Depot-ID, opsiclientd-Version |
| `getKioskProductInfosForClient` | Produktkatalog + Installationsstatus |
| `setProductActionRequestWithDependencies` | setup/uninstall vormerken (inkl. Abhängigkeiten) |
| `processActionRequests` | vorgemerkte Aktionen sofort ausführen |

Ist kein opsiclientd erreichbar (Entwicklungsrechner), fällt das Backend automatisch in einen
**Demo-Modus** mit Mock-Katalog zurück (erzwingbar via `OPSI_KIOSK_MOCK=1`).

## Voraussetzungen auf dem Client

1. **opsi-client-agent 4.3** installiert (opsiclientd-Dienst läuft).
2. **Kiosk-API aktiv** — Standard. Prüfen in `opsiclientd.conf`:
   ```ini
   [control_server]
   kiosk_api_active = true
   ```
3. **Software-on-Demand konfiguriert** (serverseitig, pro Client oder global):
   - Produktgruppe anlegen (z. B. `software-on-demand`) und Produkte zuordnen
   - Config `software-on-demand.product-group-ids` auf diese Gruppe(n) setzen
   - Damit „Jetzt ausführen" sofort wirkt: Event `on_demand` muss aktiv sein (Standard)
4. **Diese App installieren** — sinnvollerweise als opsi-Produkt paketiert (NSIS/MSI liegt
   nach `pnpm tauri:build` unter `src-tauri/target/release/bundle/`).

Keine Admin-Rechte, keine Zugangsdaten nötig: Die Kiosk-API ist von localhost ohne
Authentifizierung erreichbar und zeigt ausschließlich die per Software-on-Demand
freigegebenen Produkte.

## Produkt-Icons

Die Kiosk-JSON-RPC-API liefert **keine** Icons (der offizielle Kiosk lädt sie als Dateien vom
Depot). Diese App sucht Icons stattdessen lokal, in dieser Reihenfolge:

1. Verzeichnis aus Umgebungsvariable `OPSI_KIOSK_ICON_DIR`
2. `%ProgramData%\opsi-kiosk\icons\`
3. `icons\` neben der EXE

Dateinamenskonvention: `<productId>.png` (auch `.ico`, `.jpg`, `.svg`, `.webp`).
Das opsi-Paket, das die App verteilt, kann die Icons einfach mitliefern. Ohne Icon wird ein
generisches Paket-Symbol angezeigt.

## Entwicklung

```powershell
pnpm install
pnpm tauri:dev      # App mit Hot-Reload (Demo-Modus ohne opsiclientd)
pnpm check          # svelte-check
pnpm tauri:build    # Release-Bundle (NSIS + MSI)
```

## Sicherheit

- Die App nutzt ausschließlich die lokale Kiosk-API — sie kann nur sehen und auslösen, was
  serverseitig per Software-on-Demand freigegeben wurde.
- TLS-Zertifikatsprüfung ist für `localhost:4441` deaktiviert (opsiclientd nutzt ein
  selbstsigniertes Zertifikat); es wird nie ein entfernter Host kontaktiert.
- Produkt-IDs werden vor Pfad-Verwendung (Icon-Lookup) gegen Traversal gefiltert.
