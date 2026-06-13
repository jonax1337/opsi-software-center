//! Read-only device + account facts for the Info tab. Everything runs in the
//! logged-in standard user's context: hardware via CIM/WMI, account via an
//! [ADSISearcher] query against the current domain (no RSAT, no admin). The
//! PowerShell script is hard-coded — no user input ever reaches it.

const SYSINFO_PS: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$cs   = Get-CimInstance Win32_ComputerSystem
$bios = Get-CimInstance Win32_BIOS
$os   = Get-CimInstance Win32_OperatingSystem
$cpu  = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
$disk = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='C:'"
$ip   = [System.Net.Dns]::GetHostAddresses([System.Net.Dns]::GetHostName()) |
        Where-Object { $_.AddressFamily -eq 'InterNetwork' } |
        Select-Object -First 1 -ExpandProperty IPAddressToString

$pc = [ordered]@{
  hostname     = $env:COMPUTERNAME
  domain       = $cs.Domain
  manufacturer = "$($cs.Manufacturer)".Trim()
  model        = "$($cs.Model)".Trim()
  serial       = "$($bios.SerialNumber)".Trim()
  os           = $os.Caption
  build        = "$($os.Version) (Build $($os.BuildNumber))"
  cpu          = $cpu
  ramGb        = [math]::Round($cs.TotalPhysicalMemory / 1GB, 1)
  diskFreeGb   = [math]::Round($disk.FreeSpace / 1GB, 0)
  diskTotalGb  = [math]::Round($disk.Size / 1GB, 0)
  ip           = $ip
  lastBoot     = $os.LastBootUpTime.ToString('o')
}

$user = $null
try {
  $s = [ADSISearcher]"(&(objectCategory=user)(sAMAccountName=$env:USERNAME))"
  [void]$s.PropertiesToLoad.AddRange(@(
    'displayName','mail','department','physicalDeliveryOfficeName','title',
    'msDS-UserPasswordExpiryTimeComputed'))
  $r = $s.FindOne()
  if ($r) {
    $p = $r.Properties
    $exp = $null
    $raw = $p['msds-userpasswordexpirytimecomputed'][0]
    if ($raw -and "$raw" -ne '9223372036854775807') {
      $exp = [datetime]::FromFileTime([int64]$raw).ToString('o')
    }
    $user = [ordered]@{
      displayName     = [string]$p['displayname'][0]
      username        = "$env:USERDOMAIN\$env:USERNAME"
      mail            = [string]$p['mail'][0]
      department      = [string]$p['department'][0]
      office          = [string]$p['physicaldeliveryofficename'][0]
      title           = [string]$p['title'][0]
      passwordExpires = $exp
    }
  }
} catch {}

$diag = [ordered]@{
  logonServer  = $env:LOGONSERVER
  lastGpo      = $null
  lastUpdate   = $null
  lastUpdateId = $null
}
try {
  $e = Get-WinEvent -LogName 'Microsoft-Windows-GroupPolicy/Operational' -MaxEvents 1 -ErrorAction Stop
  if ($e) { $diag.lastGpo = $e.TimeCreated.ToString('o') }
} catch {}
try {
  $hf = Get-HotFix -ErrorAction Stop |
        Where-Object { $_.InstalledOn } |
        Sort-Object InstalledOn -Descending | Select-Object -First 1
  if ($hf) { $diag.lastUpdate = $hf.InstalledOn.ToString('o'); $diag.lastUpdateId = $hf.HotFixID }
} catch {}

[pscustomobject]@{ pc = $pc; user = $user; diag = $diag } | ConvertTo-Json -Depth 4 -Compress
"#;

/// PC hardware facts plus the signed-in user's AD attributes, as a JSON object
/// `{ pc: {...}, user: {...}|null }`. `user` is null off-domain or when the
/// directory query fails. Returned as a passthrough value — the frontend owns
/// the shape (see bridge.ts `SystemInfo`).
#[tauri::command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(|| {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
            let ps = format!(r"{windir}\System32\WindowsPowerShell\v1.0\powershell.exe");
            let out = std::process::Command::new(ps)
                .args(["-NoProfile", "-NonInteractive", "-Command", SYSINFO_PS])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| e.to_string())?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            serde_json::from_str(stdout.trim()).map_err(|e| {
                format!("invalid system-info JSON: {e} — output: {}", stdout.trim())
            })
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        Ok(serde_json::json!({ "pc": null, "user": null }))
    }
}
