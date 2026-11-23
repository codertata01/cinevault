mod commands;
mod data_sources;
mod db;
mod duplicates;
mod path_policy;
mod scanner;
mod state;
mod volumes;

use state::DbState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            path_policy::load_dev_reference_libraries();
            let conn = db::initialize(app.handle())?;
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::schema_version,
            commands::sync_connected_volumes,
            commands::list_volumes,
            commands::apply_volume_remap,
            commands::list_data_sources,
            commands::add_data_source_from_path,
            commands::remove_data_source,
            commands::scan_data_source,
            commands::scan_all_data_sources,
            commands::delete_duplicate_copy,
            commands::validate_delete_target,
            commands::list_movies,
            commands::query_movies,
            commands::list_drive_letter_counts,
            commands::export_movies_csv,
            commands::library_counts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}