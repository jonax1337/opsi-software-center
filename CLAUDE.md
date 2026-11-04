# Software Portal (opsi-kiosk) — Claude Code Context

Tauri 2 + Svelte 5 desktop app: lightweight replacement for the opsi-client-kiosk.
Talks to the local opsiclientd kiosk JSON-RPC endpoint (`https://localhost:4441/kiosk`,
unauthenticated from localhost). UI stack copied from `E:\DEV\reclaim` (same tokens,
primitives, conventions). Company-Portal-style: app tile grid → click → detail page
(description + install/uninstall there, not in the grid). `--primary`/`--ring` are
overridden at runtime with the **Windows accent color** (DWM registry via
`get_accent_color`, Dark1 for light mode and Light2 for dark mode — see
`theme.svelte.ts`); the static Fluent blue in app.css is only the fallback.
Light/dark follows the OS automatically (no manual toggle). The Soft Stack identity
(three blue app cards with a square cutout) is maintained under
`docs/branding/soft-stack/`; native window/taskbar/installer icons are the plate-less mark
filling the tile (`svg/native-icon.svg` → `tauri icon`), the tray uses `icons/tray.png`.

## Stack

| Layer | Tech |
|---|---|
| Frontend | Svelte 5 (runes only), TypeScript strict, Vite 6, Tailwind v4, bits-ui |
| Rust | Tauri 2 + reqwest (rustls, `danger_accept_invalid_certs` for localhost self-signed) |
| Persistence | localStorage (theme + `autostart-initialized` flag) |
| Window | Mica via `windowEffects`, `decorations:false` + own Company-Portal title bar (`TitleBar.svelte`, caption buttons via `windowControls`), starts hidden (`visible:false`), port 1430 |
| Tray | Permanent tray icon; single-instance; autostart at login (`--minimized`); native notifications |

## Architecture

### Rust (`src-tauri/src/`)
- **`kiosk.rs`** — JSON-RPC client + commands `kiosk_status`, `kiosk_get_products`,
  `kiosk_set_action` (typed `ActionRequest` enum), `kiosk_process_actions`. The backend
  (`Live { client_id }` or `Mock`) is cached only once resolved successfully — an unreachable
  opsiclientd is NOT cached, so a late-starting service (autostart at login) is picked up on
  the next call. **Mock only via `OPSI_KIOSK_MOCK=1` or in debug builds** (`cfg!(debug_assertions)`);
  release builds report `connected:false` + `error` instead of showing fake products.
- **`mock.rs`** — demo catalog (exact `getKioskProductInfosForClient` shape); requests
  complete after 4 s.
- **`icons.rs`** — `kiosk_get_icons`, file lookup (see Kiosk API notes).
- **`config.rs`** — `%ProgramData%\opsi-kiosk` (`data_dir()`), `get_branding` (reads
  `HKLM\SOFTWARE\opsi-kiosk\Branding`, written by the opsi package from product properties — HKLM so standard users can't plant fake support links: company name +
  support phone/email/url + `featured_products`, the comma-joined multi-value property for the
  home "Ausgewählte Apps" banner; demo mode falls back to `mock::DEMO_FEATURED`), `open_support_contact(channel)` — the frontend only names the
  channel, the URI always comes from the config (mailto:/tel:/http(s) only).
- **`device.rs`** — gpupdate, flushdns, gpresult report, pending-reboot check, restart,
  Explorer restart (own session only). **`cleanup.rs`** — %TEMP% cleanup (files untouched
  ≥ 24 h, links skipped). **`diagnostics.rs`** — support ZIP on the (known-folder) desktop via
  System32 `tar.exe`.
- **`actions.rs`** — admin maintenance scripts ("Aktionen Ihrer IT"): manifest
  `%ProgramFiles%\Software Portal\actions\actions.json` (filled by the opsi package from
  `CLIENT_DATA\actions`), run as the user (.ps1/.cmd/.bat, 10-min timeout). The UI only sends
  the action id; the script path is always re-resolved from the manifest and must be a bare
  file name (validated). Last stdout line = success message.
- **`inventory.rs`** — `get_system_info` (CIM/WMI + ADSI via a hard-coded PowerShell script).
- **`winproc.rs`** — `windows_tool()` (absolute %WINDIR% path, CREATE_NO_WINDOW),
  `shell_open()` (ShellExecuteW, never explorer.exe), `blocking()` — use these for every external process.
- **`system.rs`** — `get_accent_color`. **`tray.rs`** — tray icon/menu, `show_main()`; the tray
  uses the freestanding brand mark `icons/tray.png` (brand rule: no carrier plate at 16 px).
- **`lib.rs`** — plugin + command registration, close-to-tray, autostart flag.

### Frontend (`src/`)
- **`lib/kiosk/types.ts`** — `KioskProduct` mirrors the exact record shape of
  `getKioskProductInfosForClient` (source: opsiconfd `ext_kiosk.py`).
- **`lib/kiosk/product.ts`** — pure product rules (displayName, isPending, productState,
  filter/search/sort). Unit-tested (`product.test.ts`). Put new product logic here, not in
  components.
- **`lib/kiosk/bridge.ts`** — typed invoke() wrappers. No raw invoke elsewhere.
- **`lib/kiosk/store.svelte.ts`** — single `kiosk` store: products, status, branding,
  `connected`, search/filter, pending, `retry()`; `runAction`/`updateAll`/`sync` poll every 5 s
  until no pending requests remain (30 min timeout). 5-min background refresh reconnects when
  offline and announces new assignments/updates.
- **`lib/format.ts`** — German date/uptime/password-expiry formatting (pure, tested).
- **`App.svelte`** — Company-Portal layout, no router: `view` state
  (`home | apps | downloads | devices | help | settings`) + `selectedId` for the
  detail page, plus a `history` stack for the title bar's back button. Accent-colored `TitleBar`
  on top (search box there — typing jumps to the Apps results), compact NavPane icon rail below:
  the hamburger widens it inline into the full pane (icons keep their position, labels fade
  in); the selection pill glides within a group and shrinks/grows when moving between the
  top items and the footer.
- **`lib/views/`** — `HomeView` (app cards, notifications, "Ihre Geräte" card, "Ausgewählte
  Apps" banner, opt-in via the `featured_products` property), `AppsView` ("Anzeigen"/"Sortieren
  nach" ComboBoxes, tiles/list toggle persisted in localStorage `apps-layout`),
  `DownloadsView` (queue / updates / installed tables with "…" menu), `DeviceView` ("Geräte": computer
  details, maintenance, `SupportTools`, `AdminActions`), `HelpView` (IT contact from branding, account info),
  `SettingsView` (sync, autostart, connection, version). The device/account inventory is
  loaded once via `lib/system/systemInfo.svelte.ts`. Visual language follows Company Portal:
  sentence-case 20px section headings, no monospace in product UI, left-aligned pages.
- **`lib/components/`** — `CatalogGate` (loading / browser preview / "Keine Verbindung" with
  retry / demo banner around catalog views), `ProductCollection` (cards or table),
  `ProductTile` (CP app card), `ProductTable` (CP table: Name/Version/Status +
  actions snippet), `ProductStatus` (one status line for both), `AppIcon`, `ProductDetail` +
  `ProductActions`, `TitleBar`.
- **Actions execute immediately** (no "Vormerken"): `runAction` = setAction +
  processActionRequests([id]). `sync()` = refresh + processActionRequests(null) (ALL pending,
  e.g. set server-side via AD-group automation) + gpupdate. Completion toasts name the
  products (never a generic "alle Aktionen").
- **Sort**: home sections put products with available updates first, rest alphabetical (de
  locale); the Apps page uses the "Sortieren nach" picker (`kiosk.sort`, name A–Z / Z–A).
- **Tray/autostart/single-instance** — the app lives in the tray: `CloseRequested` only
  hides; quitting only via tray "Beenden". Single-instance plugin registered FIRST.
  Autostart (HKCU Run, `--minimized`) is enabled once on first run
  (`autostart-initialized` localStorage flag), then respects the user's choice.
  `lib/system/desktop.ts` wraps autostart, notifications and the tray event; all no-ops in
  the browser preview.
- **`lib/ui/`** — primitives copied from reclaim (see reclaim's `docs/DESIGN_SYSTEM.md`).

### Deployment (`deploy/`, `.github/workflows/build.yml`)
- `deploy/opsi/` — opsi package sources; product properties `company_name`, `support_*`
  → `HKLM\SOFTWARE\opsi-kiosk\Branding`; the package recreates `%ProgramData%\opsi-kiosk` with an
  ACL (SYSTEM/Administrators write, Users read). `deploy/build-opsi-package.ps1` builds the release, syncs the version
  from `package.json` into `control` + `setup.opsiscript`, and packs
  `dist-opsi/software-portal_<v>-opsi-source.tar.gz`. Tag `v*` → CI builds + releases it.
- Version lives in `package.json` (tauri.conf.json points at it) and `Cargo.toml`.
- **GitHub**: https://github.com/jonax1337/opsi-software-portal (private).
  The original stock app icon has been replaced by the Soft Stack identity.

## Conventions (inherited from reclaim)

1. Svelte 5 runes only — no `export let`, no legacy stores.
2. **German UI strings** (end-user audience), English code + comments.
3. Tokens, not raw colors. Surface ladder (`bg-surface-1..4`, `border-hairline`).
4. ListCard for row lists, `gap-0 py-0` baked in. EmptyState for loading/empty.
5. Every Rust command gets a typed bridge.ts wrapper.
6. Icons only from `$lib/ui/icons` (Microsoft Fluent UI System Icons via unplugin-icons);
   use the variant of the rendered size (16px in controls, 20px in the nav rail).

## Kiosk API notes

- JSON-RPC positional params: `{"id":n,"method":"...","params":[...]}`.
- `getKioskProductInfosForClient(clientId, addConfigs=false)` → list of product records.
- `processActionRequests([productIds])` fires the `on_demand` event server-side; progress
  is observed by re-polling product infos (actionRequest clears when done).
- Icons are NOT in the API — `kiosk_get_icons` reads local files from
  `OPSI_KIOSK_ICON_DIR` / `%ProgramData%\opsi-kiosk\icons` / `icons` next to the exe
  (`<productId>.png|ico|jpg|svg|webp`), returns data-URLs.
- `time` crate is pinned to 0.3.47 in Cargo.lock (0.3.48 breaks cookie 0.18 / tauri).

## Commands

- `pnpm tauri:dev` — app with hot reload (mock mode on dev machines)
- `pnpm check` — svelte-check, must be 0 errors
- `pnpm test` — vitest (pure logic in `product.ts`, `format.ts`)
- `cargo test` (from `src-tauri/`) — Rust unit tests
- `cargo check` (from `src-tauri/`) — Rust validate
- `pnpm tauri:build` — NSIS + MSI bundles

## Deployment intent

Packaged as an opsi product (localboot), rolled out via opsi itself. The opsi package can
ship product icons into `%ProgramData%\opsi-kiosk\icons\`.
