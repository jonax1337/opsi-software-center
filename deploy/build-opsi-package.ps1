<#
.SYNOPSIS
    Builds the ready-to-upload opsi package source for "software-center".

.DESCRIPTION
    1. Builds the Tauri release (NSIS installer), unless -SkipBuild.
    2. Copies the installer of the package.json version to CLIENT_DATA\files.
    3. Writes that version into OPSI\control and setup.opsiscript.
    4. Optionally copies product icons from -IconSource to CLIENT_DATA\icons
       and maintenance actions (actions.json + scripts) from -ActionSource to
       CLIENT_DATA\actions.
    5. Packs deploy\opsi into dist-opsi\software-center_<version>-opsi-source.tar.gz
       (forward-slash paths, extracts cleanly on the Linux opsi server).

.EXAMPLE
    .\deploy\build-opsi-package.ps1
.EXAMPLE
    .\deploy\build-opsi-package.ps1 -SkipBuild -IconSource \\depot\opsi_depot\opsi-client-kiosk\files\app\ock_custom\product_icons
.EXAMPLE
    .\deploy\build-opsi-package.ps1 -SkipBuild -ActionSource .\deploy\opsi\actions-example
#>
param(
    [switch]$SkipBuild,
    [string]$IconSource,
    [string]$ActionSource
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$opsiDir = Join-Path $root "deploy\opsi"
$distDir = Join-Path $root "dist-opsi"
$version = (Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json).version
# Tauri's NSIS naming: <productName>_<version>_<arch>-setup.exe
$installer = Join-Path $root "src-tauri\target\release\bundle\nsis\Software Center_${version}_x64-setup.exe"
$utf8NoBom = New-Object System.Text.UTF8Encoding $false

function Set-VersionLine([string]$Path, [string]$Pattern, [string]$Line) {
    $text = [System.IO.File]::ReadAllText($Path)
    $regex = [regex]::new($Pattern, "Multiline")
    if (-not $regex.IsMatch($text)) { throw "Versionszeile nicht gefunden in $Path" }
    # '$' is special in .NET replacement strings
    $updated = $regex.Replace($text, $Line.Replace('$', '$$'), 1)
    [System.IO.File]::WriteAllText($Path, $updated, $utf8NoBom)
}

if (-not $SkipBuild) {
    Write-Host "Baue Release $version ..." -ForegroundColor Cyan
    Push-Location $root
    try {
        pnpm tauri:build
        if ($LASTEXITCODE -ne 0) { throw "pnpm tauri:build fehlgeschlagen (ExitCode $LASTEXITCODE)" }
    } finally {
        Pop-Location
    }
}
if (-not (Test-Path -LiteralPath $installer)) {
    throw "Installer fuer Version $version fehlt: $installer (ohne -SkipBuild ausfuehren)"
}

$filesDir = Join-Path $opsiDir "CLIENT_DATA\files"
$iconsDir = Join-Path $opsiDir "CLIENT_DATA\icons"
$actionsDir = Join-Path $opsiDir "CLIENT_DATA\actions"
New-Item -ItemType Directory -Force $filesDir, $iconsDir, $actionsDir, $distDir | Out-Null

Copy-Item -LiteralPath $installer (Join-Path $filesDir "software-center-setup.exe") -Force
Write-Host "Installer:  $(Split-Path $installer -Leaf) -> CLIENT_DATA\files\software-center-setup.exe" -ForegroundColor Green

Set-VersionLine (Join-Path $opsiDir "OPSI\control") '^version: [^\r\n]*(?=\r?\n\s*priority:)' "version: $version"
Set-VersionLine (Join-Path $opsiDir "CLIENT_DATA\setup.opsiscript") '^Set \$ProductVersion\$ = "[^"]*"' "Set `$ProductVersion`$ = `"$version`""
Write-Host "Version:    $version -> OPSI\control, setup.opsiscript" -ForegroundColor Green

if ($IconSource) {
    $icons = Get-ChildItem -LiteralPath $IconSource -File |
        Where-Object { $_.Extension -in ".png", ".ico", ".jpg", ".jpeg", ".svg", ".webp" }
    if ($icons) { $icons | Copy-Item -Destination $iconsDir -Force }
    Write-Host "Icons:      $($icons.Count) Datei(en) aus $IconSource" -ForegroundColor Green
} elseif (-not (Get-ChildItem $iconsDir -File)) {
    Write-Host "Icons:      keine (optional, siehe README -> Produkt-Icons)" -ForegroundColor Yellow
}

if ($ActionSource) {
    if (-not (Test-Path -LiteralPath (Join-Path $ActionSource "actions.json"))) {
        throw "actions.json fehlt in $ActionSource"
    }
    # replace, not merge: a script removed from the source must not linger in the package
    Get-ChildItem -LiteralPath $actionsDir -File | Remove-Item -Force
    $actions = Get-ChildItem -LiteralPath $ActionSource -File |
        Where-Object { $_.Extension -in ".json", ".ps1", ".cmd", ".bat" }
    $actions | Copy-Item -Destination $actionsDir -Force
    Write-Host "Aktionen:   $($actions.Count) Datei(en) aus $ActionSource" -ForegroundColor Green
} elseif (-not (Get-ChildItem $actionsDir -File)) {
    Write-Host "Aktionen:   keine (optional, siehe README -> Wartungsaktionen)" -ForegroundColor Yellow
}

$archive = Join-Path $distDir "software-center_${version}-opsi-source.tar.gz"
# Windows' bsdtar explicitly: a GNU tar from Git would read "C:\..." as host:path
& (Join-Path $env:WINDIR "System32\tar.exe") -czf $archive -C $opsiDir OPSI CLIENT_DATA
if ($LASTEXITCODE -ne 0) { throw "tar fehlgeschlagen (ExitCode $LASTEXITCODE)" }

Write-Host "`nPaketquelle: $archive" -ForegroundColor Green
Write-Host "Auf dem opsi-Server:"
Write-Host "  mkdir -p /var/lib/opsi/workbench/software-center && tar -xzf software-center_${version}-opsi-source.tar.gz -C /var/lib/opsi/workbench/software-center"
Write-Host "  cd /var/lib/opsi/workbench/software-center && opsi-makepackage && opsi-package-manager -i software-center_${version}-1.opsi"
