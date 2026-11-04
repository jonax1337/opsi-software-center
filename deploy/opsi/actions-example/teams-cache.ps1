# Beispielaktion fuer das Software Portal: Cache des neuen Microsoft Teams leeren.
# Laeuft als angemeldeter Benutzer. Exit-Code 0 = Erfolg; die letzte Ausgabezeile
# erscheint im Software Portal als Meldung, bei Fehlern die erste Fehlerzeile.
$ErrorActionPreference = "Stop"

$cache = Join-Path $env:LOCALAPPDATA "Packages\MSTeams_8wekyb3d8bbwe\LocalCache\Microsoft\MSTeams"
if (-not (Test-Path -LiteralPath $cache)) {
    Write-Output "Kein Teams-Cache gefunden – nichts zu tun."
    exit 0
}

Get-Process -Name "ms-teams" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3
Remove-Item -LiteralPath $cache -Recurse -Force

Start-Process "shell:AppsFolder\MSTeams_8wekyb3d8bbwe!MSTeams"
Write-Output "Teams-Cache geleert, Teams startet neu."
