# Collects the NSIS installer (and optional product icons) into the opsi
# package source under deploy/opsi/CLIENT_DATA. Run from anywhere.
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent

$setup = Get-ChildItem "$root\src-tauri\target\release\bundle\nsis\*-setup.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $setup) {
    Write-Host "Kein NSIS-Setup gefunden - baue Release..." -ForegroundColor Yellow
    Push-Location $root
    pnpm tauri:build
    Pop-Location
    $setup = Get-ChildItem "$root\src-tauri\target\release\bundle\nsis\*-setup.exe" | Select-Object -First 1
}

$files = "$root\deploy\opsi\CLIENT_DATA\files"
$icons = "$root\deploy\opsi\CLIENT_DATA\icons"
New-Item -ItemType Directory -Force $files, $icons | Out-Null

Copy-Item $setup.FullName "$files\software-center-setup.exe" -Force
Write-Host "Installer: $($setup.Name) -> CLIENT_DATA\files\software-center-setup.exe" -ForegroundColor Green

if (-not (Get-ChildItem $icons -ErrorAction SilentlyContinue)) {
    Write-Host "Hinweis: CLIENT_DATA\icons ist leer - Produkt-Icons vom Depot dazulegen:" -ForegroundColor Yellow
    Write-Host "  /var/lib/opsi/depot/opsi-client-kiosk/files/app/ock_custom/product_icons/*"
}

Write-Host "`nPaketquelle bereit: $root\deploy\opsi" -ForegroundColor Green
Write-Host "Naechste Schritte: auf den opsi-Server kopieren, dann 'opsi-makepackage' + 'opsi-package-manager -i'"
