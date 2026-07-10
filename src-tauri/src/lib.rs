mod esi;
mod evesettings;
mod updates;

use evesettings::{
    analyze_import, copy_settings, copy_settings_selective, create_backup, delete_backup,
    delete_backups, execute_import, export_settings, get_app_data, get_entry_display_name,
    open_formation_editor, read_probe_formations, set_alias, set_brackets_always_show,
    write_probe_formations,
};
use updates::{check_for_update, get_app_info};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_app_data,
            read_probe_formations,
            write_probe_formations,
            open_formation_editor,
            get_entry_display_name,
            create_backup,
            delete_backup,
            delete_backups,
            copy_settings,
            copy_settings_selective,
            set_alias,
            set_brackets_always_show,
            check_for_update,
            get_app_info,
            export_settings,
            analyze_import,
            execute_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
