# opsi-Paket: software-center

Paketquellen für das Localboot-Produkt `software-center`. Ersetzt den
Standard-`opsi-client-kiosk` durch das Software Center.

## Paket bauen

1. **Installer einsammeln** (auf der Dev-Maschine):
   ```powershell
   .\deploy\build-opsi-package.ps1
   ```
   Das Skript baut das Release (falls nötig), kopiert den NSIS-Installer nach
   `CLIENT_DATA\files\software-center-setup.exe` und die Produkt-Icons vom Depot
   (falls vorhanden) nach `CLIENT_DATA\icons\`.

2. **Auf den opsi-Server kopieren** (z. B. nach `/home/opsiproducts/software-center`):
   ```
   deploy/opsi/  →  /home/opsiproducts/software-center/
   ```

3. **Paket bauen + installieren** (auf dem opsi-Server):
   ```bash
   cd /home/opsiproducts/software-center
   opsi-makepackage
   opsi-package-manager -i software-center_0.1.0-1.opsi
   ```

## Produkt-Icons

`CLIENT_DATA\icons\<productId>.png` wird bei der Installation nach
`%ProgramData%\opsi-kiosk\icons\` kopiert. Quelle ist sinnvollerweise derselbe
Bestand wie beim alten Kiosk:
```
/var/lib/opsi/depot/opsi-client-kiosk/files/app/ock_custom/product_icons/
```
Zusätzlich liest die App Icons zur Laufzeit direkt vom Depot-Share — die lokale
Kopie ist nur Cache/Fallback (z. B. für Offline-Clients).

## Standard-Kiosk ablösen

Das Software Center ersetzt den `opsi-client-kiosk`. Rollout-Reihenfolge:

1. `software-center` auf den Zielclients auf **setup** stellen (bzw. das Produkt
   in die entsprechende ITGG-SW-Produktgruppe aufnehmen, dann übernimmt die
   Automatik).
2. `opsi-client-kiosk` auf **uninstall** stellen, z. B. für alle Clients mit
   installiertem Kiosk:
   ```bash
   opsi-admin -d method productOnClient_getObjects '[]' '{"productId":"opsi-client-kiosk","installationStatus":"installed"}' \
     | jq -r '.[].clientId' \
     | while read c; do
         opsi-admin -d method setProductActionRequest "opsi-client-kiosk" "$c" "uninstall"
       done
   ```
3. Wichtig: Den Config-Wert `software-on-demand.product-group-ids` und das
   `on_demand`-Event **nicht** entfernen — das Software Center nutzt dieselbe
   Software-on-Demand-Mechanik wie der alte Kiosk, nur das UI ist neu.

## Hinweise

- Der NSIS-Installer ist auf `perMachine` konfiguriert und läuft sauber im
  SYSTEM-Kontext des opsi-client-agent (`/S` = silent).
- Bei App-Updates: neue Version bauen, `build-opsi-package.ps1` ausführen,
  `version:` in `OPSI/control` hochziehen, Paket neu bauen.
