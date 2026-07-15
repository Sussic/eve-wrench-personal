mod esi;
mod evesettings;
mod updates;

use evesettings::{
    analyze_import, copy_probe_formations_varied, copy_settings, copy_settings_selective,
    create_backup, create_recovery_snapshot, delete_backup, delete_backups, execute_import,
    export_probe_formation, export_probe_formations, export_settings, get_app_data,
    get_entry_display_name, import_probe_formation, is_eve_running, open_formation_editor,
    read_probe_formations, restore_recovery_snapshot, set_alias, set_brackets_always_show,
    write_probe_formations,
};
use updates::get_app_info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_app_data,
            is_eve_running,
            read_probe_formations,
            write_probe_formations,
            copy_probe_formations_varied,
            import_probe_formation,
            export_probe_formation,
            export_probe_formations,
            open_formation_editor,
            get_entry_display_name,
            create_backup,
            delete_backup,
            delete_backups,
            copy_settings,
            copy_settings_selective,
            set_alias,
            set_brackets_always_show,
            get_app_info,
            export_settings,
            create_recovery_snapshot,
            restore_recovery_snapshot,
            analyze_import,
            execute_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
