mod scan;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(scan::scan_commands::ScanState::default())
        .invoke_handler(tauri::generate_handler![
            scan::scan_commands::file_types,
            scan::scan_commands::scan_folder,
            scan::scan_commands::cancel_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Diskmora");
}
