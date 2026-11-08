pub mod commands;
pub mod detect;
pub mod integrity;
pub mod model;
pub mod release;
pub mod util;

use std::sync::Arc;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Arc::new(commands::AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::get_install_info,
            commands::get_manager_info,
            commands::check_update,
            commands::verify_installation,
            commands::open_external,
            commands::copy_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to start RuxCmd Desktop");
}
