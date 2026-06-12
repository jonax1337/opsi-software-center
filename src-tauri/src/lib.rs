mod kiosk;

pub fn run() {
    tauri::Builder::default()
        .manage(kiosk::KioskState::new())
        .invoke_handler(tauri::generate_handler![
            kiosk::kiosk_status,
            kiosk::kiosk_get_products,
            kiosk::kiosk_set_action,
            kiosk::kiosk_process_actions,
            kiosk::kiosk_get_icons,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
