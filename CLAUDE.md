# Software Center (opsi-kiosk) — Claude Code Context

Tauri 2 + Svelte 5 desktop app: lightweight replacement for the opsi-client-kiosk.
Talks to the local opsiclientd kiosk JSON-RPC endpoint (`https://localhost:4441/kiosk`,
unauthenticated from localhost). UI stack copied from `E:\DEV\reclaim` (same tokens,
primitives, conventions). Company-Portal-style: app tile grid → click → detail page
(description + install/uninstall there, not in the grid). `--primary`/`--ring` are
overridden at runtime with the **Windows accent color** (DWM registry via
`get_accent_color`, hue extracted, L/C normalized — see `theme.svelte.ts`); the static
opsi magenta (#CB1D58 = oklch(0.549 0.205 9)) in app.css is only the fallback.
Light/dark follows the OS automatically (no manual toggle). The Soft Stack identity
(three blue app cards with a square cutout) is maintained under
`docs/branding/soft-stack/`; native window/tray/installer icons use this identity.

## Stack

| Layer | Tech |
|---|---|
| Frontend | Svelte 5 (runes only), TypeScript strict, Vite 6, Tailwind v4, bits-ui |
| Rust | Tauri 2 + reqwest (rustls, `danger_accept_invalid_certs` for localhost self-signed) |
| Persistence | localStorage (theme + `autostart-initialized` flag) |
| Window | Mica via `windowEffects`, native decorations, starts hidden (`visible:false`), port 1430 |
| Tray | Permanent tray icon; single-instance; autostart at login (`--minimized`); native notifications |

## Architecture

- **`src-tauri/src/kiosk.rs`** — everything backend. `KioskState` resolves once to
  `Backend::Live { client_id }` (opsiclientd reachable) or `Backend::Mock` (in-memory
  catalog, actions complete after 4 s — full UI flow testable without opsi).
  `OPSI_KIOSK_MOCK=1` forces mock. Commands: `kiosk_status`, `kiosk_get_products`,
  `kiosk_set_action`, `kiosk_process_actions`, `kiosk_get_icons`.
- **`src/lib/kiosk/types.ts`** — `KioskProduct` mirrors the exact record shape of
  `getKioskProductInfosForClient` (source: opsiconfd `ext_kiosk.py`).
- **`src/lib/kiosk/bridge.ts`** — typed invoke() wrappers. No raw invoke elsewhere.
- **`src/lib/kiosk/store.svelte.ts`** — single `kiosk` store: products, status, search,
  filter, pending (= products with actionRequest set), executeAll → poll every 5 s until
  no pending requests remain (30 min timeout). Icons loaded once per session.
- **`src/lib/components/ProductTile.svelte`** — grid tile: icon, name, status line
  (installed / update / pending / running). Click → detail.
- **`src/lib/components/ProductDetail.svelte`** — detail page: back link, hero (icon,
  name, version, badges, action buttons), description card, details grid (DataField).
  Action buttons call `kiosk.setAction` (= opsi `setProductActionRequestWithDependencies`).
- **`src/App.svelte`** — single page, no router; `selectedId` state switches grid ↔
  detail. Titlebar + PageHeader + SearchInput + FilterChips + tile grid + StatusFooter.
- **`src/lib/components/StatusFooter.svelte`** — classic status bar (bottom): connection
  dot + clientId, depot, app count, last-sync time, Sync button. No pills in the titlebar.
- **Actions execute immediately** (no "Vormerken"): `runAction` = setAction +
  processActionRequests([id]) in one go. `sync()` = refresh + processActionRequests(null)
  (= ALL pending requests, e.g. set server-side via AD-group→opsi automation). A 5-min
  background refresh detects new server-side assignments and offers a "Jetzt ausführen"
  toast. `updateAll()` queues setup for every updatable product. Completion toasts name
  the products (pollUntilDone tracks a `watched` map — never a generic "alle Aktionen").
- **Grid sort**: products with available updates first, rest alphabetical (de locale).
- **`deploy/opsi/`** — opsi package sources (control + opsiscripts). `deploy/
  build-opsi-package.ps1` collects the NSIS installer into CLIENT_DATA. NSIS is
  configured perMachine so it installs in the opsi-client-agent's SYSTEM context.
  The package replaces the standard opsi-client-kiosk (uninstall via opsi, see
  deploy/opsi/README.md).
- **GitHub**: https://github.com/jonax1337/opsi-software-center (private).
  The original stock app icon has been replaced by the Soft Stack identity.
- **`src-tauri/src/tray.rs`** — system-tray icon + menu (Öffnen / Nach Updates suchen /
  Beenden). Left-click reveals the window; "Nach Updates suchen" emits the
  `tray://check-updates` event (store listens → `sync()`). `show_main()` is the shared
  reveal helper.
- **Tray/autostart/single-instance (`lib.rs`)** — the app lives in the tray permanently:
  `CloseRequested` only `hide()`s the window (`prevent_close`), so **quitting is only via the
  tray "Beenden"** (`app.exit`). `tauri-plugin-single-instance` (registered FIRST) reveals the
  running window on a second launch. `tauri-plugin-autostart` (LaunchAgent + `--minimized`
  flag) registers per-user HKCU Run; when started with `--minimized` the window stays hidden.
  The window is `visible:false` in config and shown in `setup()` only when not autostarted (no
  flash). Notifications via `tauri-plugin-notification`.
- **`src/lib/system/desktop.ts`** — frontend wrappers for the above: `ensureAutostartDefault`
  (enables autostart once on first run, then respects manual choice via the
  `autostart-initialized` localStorage flag), `autostartEnabled`/`setAutostart` (Info →
  Einstellungen toggle), `initNotifications`/`notify` (native OS toast — used because the
  window is usually hidden in the tray), `onTrayCheckUpdates`. All no-op in browser preview.
- **Update notifications** — `store.notifyNewUpdates()` fires a native notification for any
  `updatable` product not yet announced this session (tracked in `notifiedUpdates`); called at
  startup and on every 5-min `backgroundRefresh`. New server-side assignments also get a native
  notification alongside the in-app toast.
- **`src-tauri/src/system.rs`** — `get_accent_color` (HKCU DWM AccentColor, ABGR u32).
- **`src/lib/ui/`** — primitives copied from reclaim (see reclaim's
  `docs/DESIGN_SYSTEM.md` for the full token/primitive reference). Button's
  svelte-spa-router dependency was removed; Titlebar simplified (no tray).

## Conventions (inherited from reclaim)

1. Svelte 5 runes only — no `export let`, no legacy stores.
2. **German UI strings** (end-user audience), English code + comments.
3. Tokens, not raw colors. Surface ladder (`bg-surface-1..4`, `border-hairline`).
4. ListCard for row lists, `gap-0 py-0` baked in. EmptyState for loading/empty.
5. Every Rust command gets a typed bridge.ts wrapper.

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
- `cargo check` (from `src-tauri/`) — Rust validate
- `pnpm tauri:build` — NSIS + MSI bundles

## Deployment intent

Packaged as an opsi product (localboot), rolled out via opsi itself. The opsi package can
ship product icons into `%ProgramData%\opsi-kiosk\icons\`.
