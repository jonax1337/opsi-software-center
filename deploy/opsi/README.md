# opsi-Paket `software-portal` — Interna

Installation und Rollout stehen in der [Haupt-README](../../README.md#installation-in-fünf-schritten).
Diese Datei beschreibt, was im Paket steckt und was die Skripte tun.

## Aufbau

```
deploy/opsi/
├── OPSI/control                  Produktdefinition + Properties
└── CLIENT_DATA/
    ├── setup.opsiscript          Installation (Fast-Exit, Icons, Branding in HKLM)
    ├── uninstall.opsiscript      Deinstallation
    ├── files/                    ← software-portal-setup.exe (vom Build-Skript)
    ├── icons/                    ← <productId>.png usw. (optional)
    └── actions/                  ← actions.json + Wartungsskripte (optional)
deploy/opsi/actions-example/      Vorlage für actions/ (Teams-Cache, OneDrive-Reset)
```

`files/`, `icons/` und `actions/` sind nicht eingecheckt; `deploy/build-opsi-package.ps1` befüllt sie.

## setup.opsiscript

Bei jedem Lauf:

1. legt `%ProgramData%\opsi-kiosk\` neu an und sperrt ihn per ACL: nur SYSTEM und
   Administratoren dürfen schreiben, Benutzer nur lesen (sonst könnten Standardbenutzer dort
   Dateien ablegen); kopiert anschließend `CLIENT_DATA\icons\*` nach `…\opsi-kiosk\icons\`,
2. schreibt die Produkt-Properties als REG_SZ nach `HKLM\SOFTWARE\opsi-kiosk\Branding`
   (`company_name`, `support_phone`, `support_email`, `support_url`) — HKLM, damit niemand
   außer der IT die angezeigten Support-Kontakte ändern kann.

Danach nur, wenn die installierte Version (`DisplayVersion` im Uninstall-Schlüssel
`Software Portal`) nicht der Paketversion entspricht:

3. beendet laufende Instanzen und entfernt einen Vorgänger unter dem alten Namen
   „Software Center“ (bis 0.3.0): NSIS-Uninstaller, Installationsordner und den
   Autostart-Eintrag `Software Center` im HKCU-Run-Schlüssel aller Benutzerprofile,
4. kopiert den Installer nach `%opsiTmpDir%` (ein Start direkt vom Depot-Share hängt in
   Session 0),
5. führt ihn still aus (`/S`; NSIS `perMachine`, SYSTEM-tauglich) und bricht bei ExitCode ≠ 0
   mit `isFatalError` ab,
6. entfernt den Desktop-Link; der Startmenü-Eintrag bleibt.

Immer, auch beim Fast-Exit:

7. ersetzt `%ProgramFiles%\Software Portal\actions\` durch `CLIENT_DATA\actions\*` (Wartungsaktionen),
   sofern das Paket eine `actions.json` enthält. Bewusst nicht unter `%ProgramData%`: Dort darf jeder
   Benutzer den Ordner vorab anlegen und besitzt ihn dann; unter Program Files kann er nichts anlegen.

Die Soll-Version in `setup.opsiscript` und `version:` in `control` setzt das Build-Skript aus
`package.json` — nicht von Hand pflegen.

## uninstall.opsiscript

Beendet laufende Instanzen, ruft den NSIS-Uninstaller still auf und entfernt
`%ProgramFiles%\Software Portal`, `%ProgramData%\opsi-kiosk` (Icons) und den Registry-Schlüssel `HKLM\SOFTWARE\opsi-kiosk`.

## Umstieg vom Produkt `software-center`

Bis 0.3.0 hieß das Produkt `software-center`. `software-portal` auf **setup** stellen genügt,
das Setup räumt den Vorgänger ab. Das alte Produkt danach nur aus der Client-Zuordnung bzw.
vom Server entfernen — **nicht** auf *uninstall* stellen: Dessen Skript löscht
`%ProgramData%\opsi-kiosk` und `HKLM\SOFTWARE\opsi-kiosk`, die `software-portal` weiterverwendet.

## Paketquelle bauen

```powershell
.\deploy\build-opsi-package.ps1                       # baut Release, dann Paketquelle
.\deploy\build-opsi-package.ps1 -SkipBuild            # vorhandenen Release-Build verwenden
.\deploy\build-opsi-package.ps1 -IconSource <ordner>  # Icons ins Paket übernehmen
.\deploy\build-opsi-package.ps1 -ActionSource <ordner>  # Wartungsaktionen ins Paket übernehmen
```

Ergebnis: `dist-opsi\software-portal_<version>-opsi-source.tar.gz` — auf dem Server entpacken,
`opsi-makepackage`, `opsi-package-manager -i`.

## Startmenü-Pin

Das Setup legt nur den Startmenü-Eintrag an. Echtes „An Start anheften" (Windows-11-Pins) ist
programmatisch gesperrt; per GPO `ConfigureStartPins` ist es möglich (Start-Layout-JSON mit
`"desktopAppLink": "%ALLUSERSPROFILE%\\Microsoft\\Windows\\Start Menu\\Programs\\Software Portal.lnk"`).
Achtung: Diese Richtlinie ersetzt die komplette Pin-Liste der Anwender.
