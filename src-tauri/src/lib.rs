mod config;
mod device;
mod icons;
mod inventory;
mod kiosk;
mod mock;
mod system;
mod tray;
mod winproc;

use tauri::{Manager, WindowEvent};

/// Autostart passes this flag so the very first launch after login comes up
/// silently in the tray instead of popping the window open.
const MINIMIZED_FLAG: &str = "--minimized";

pub fn run() {
    let mut builder = tauri::Builder::default();

    // Single instance MUST be the first plugin. A second launch just reveals
    // the window of the already-running instance.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::show_main(app);
        }));
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .manage(kiosk::KioskState::new())
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::MacosLauncher;
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    Some(vec![MINIMIZED_FLAG]),
                ))?;
            }

            tray::create_tray(app.handle())?;

            // Show the window unless we were started by autostart (--minimized),
            // in which case we stay in the tray until the user opens us.
            let start_hidden = std::env::args().any(|a| a == MINIMIZED_FLAG);
            if let Some(window) = app.get_webview_window("main") {
                if start_hidden {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window only hides it to the tray; quitting happens
            // exclusively through the tray menu (app.exit).
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            kiosk::kiosk_status,
            kiosk::kiosk_get_products,
            kiosk::kiosk_set_action,
            kiosk::kiosk_process_actions,
            icons::kiosk_get_icons,
            config::get_branding,
            config::open_support_contact,
            device::device_gpupdate,
            device::device_pending_reboot,
            device::device_flush_dns,
            device::device_gpresult_report,
            system::get_accent_color,
            inventory::get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
