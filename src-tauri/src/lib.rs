mod device;
mod inventory;
mod kiosk;
mod system;

pub fn run() {
    tauri::Builder::default()
        .manage(kiosk::KioskState::new())
        .invoke_handler(tauri::generate_handler![
            kiosk::kiosk_status,
            kiosk::kiosk_get_products,
            kiosk::kiosk_set_action,
            kiosk::kiosk_process_actions,
            kiosk::kiosk_get_icons,
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
