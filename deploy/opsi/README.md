# opsi-Paket `software-center` — Interna

Installation und Rollout stehen in der [Haupt-README](../../README.md#installation-in-fünf-schritten).
Diese Datei beschreibt, was im Paket steckt und was die Skripte tun.

## Aufbau

```
deploy/opsi/
├── OPSI/control                  Produktdefinition + Properties
└── CLIENT_DATA/
    ├── setup.opsiscript          Installation (Fast-Exit, Icons, Branding in HKLM)
    ├── uninstall.opsiscript      Deinstallation
    ├── files/                    ← software-center-setup.exe (vom Build-Skript)
    └── icons/                    ← <productId>.png usw. (optional)
```

`files/` und `icons/` sind nicht eingecheckt; `deploy/build-opsi-package.ps1` befüllt sie.

## setup.opsiscript

Bei jedem Lauf:

1. legt `%ProgramData%\opsi-kiosk\` neu an und sperrt ihn per ACL: nur SYSTEM und
   Administratoren dürfen schreiben, Benutzer nur lesen (sonst könnten Standardbenutzer dort
   Dateien ablegen); kopiert anschließend `CLIENT_DATA\icons\*` nach `…\opsi-kiosk\icons\`,
2. schreibt die Produkt-Properties als REG_SZ nach `HKLM\SOFTWARE\opsi-kiosk\Branding`
   (`company_name`, `support_phone`, `support_email`, `support_url`) — HKLM, damit niemand
   außer der IT die angezeigten Support-Kontakte ändern kann.

Danach nur, wenn die installierte Version (`DisplayVersion` im Uninstall-Schlüssel
`Software Center`) nicht der Paketversion entspricht:

3. beendet laufende Instanzen,
4. kopiert den Installer nach `%opsiTmpDir%` (ein Start direkt vom Depot-Share hängt in
   Session 0),
5. führt ihn still aus (`/S`; NSIS `perMachine`, SYSTEM-tauglich) und bricht bei ExitCode ≠ 0
   mit `isFatalError` ab,
6. entfernt den Desktop-Link; der Startmenü-Eintrag bleibt.

Die Soll-Version in `setup.opsiscript` und `version:` in `control` setzt das Build-Skript aus
`package.json` — nicht von Hand pflegen.

## uninstall.opsiscript

Beendet laufende Instanzen, ruft den NSIS-Uninstaller still auf und entfernt
`%ProgramFiles%\Software Center`, `%ProgramData%\opsi-kiosk` (Icons) und den Registry-Schlüssel `HKLM\SOFTWARE\opsi-kiosk`.

## Paketquelle bauen

```powershell
.\deploy\build-opsi-package.ps1                       # baut Release, dann Paketquelle
.\deploy\build-opsi-package.ps1 -SkipBuild            # vorhandenen Release-Build verwenden
.\deploy\build-opsi-package.ps1 -IconSource <ordner>  # Icons ins Paket übernehmen
```

Ergebnis: `dist-opsi\software-center_<version>-opsi-source.tar.gz` — auf dem Server entpacken,
`opsi-makepackage`, `opsi-package-manager -i`.

## Startmenü-Pin

Das Setup legt nur den Startmenü-Eintrag an. Echtes „An Start anheften" (Windows-11-Pins) ist
programmatisch gesperrt; per GPO `ConfigureStartPins` ist es möglich (Start-Layout-JSON mit
`"desktopAppLink": "%ALLUSERSPROFILE%\\Microsoft\\Windows\\Start Menu\\Programs\\Software Center.lnk"`).
Achtung: Diese Richtlinie ersetzt die komplette Pin-Liste der Anwender.
