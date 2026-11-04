//! System-tray icon. The app lives in the tray permanently: closing the window
//! only hides it (see `lib.rs` window-event handler), so the tray menu is the
//! single way to bring it back or quit for good.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

/// Event emitted to the frontend when the user asks the tray to look for
/// updates. The store listens for it and runs a sync.
pub const CHECK_UPDATES_EVENT: &str = "tray://check-updates";

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Software Portal öffnen", true, None::<&str>)?;
    let check = MenuItem::with_id(app, "check", "Nach Updates suchen", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &check, &sep, &quit])?;

    TrayIconBuilder::with_id("main")
        // Brand rule: at tray size use the freestanding Soft Stack mark, not
        // the app icon with its light carrier plate. 32 px stays crisp at 200 %.
        .icon(tauri::include_image!("icons/tray.png"))
        .tooltip("Software Portal")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "check" => {
                show_main(app);
                let _ = app.emit(CHECK_UPDATES_EVENT, ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // single left click brings the window back, mirroring Windows tray apps
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Reveal and focus the main window (it may be hidden in the tray).
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
