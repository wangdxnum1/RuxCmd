pub mod commands;
pub mod detect;
pub mod integrity;
pub mod model;
pub mod release;
pub mod update;
pub mod util;

use std::sync::Arc;
use tauri::Manager;

/// The layout is tuned for this exact client size and the window is not
/// resizable. Below it the About view would need a scrollbar, so anything
/// smaller falls back to fitting the monitor's work area instead.
const DESIGN_WIDTH: f64 = 960.0;
const DESIGN_HEIGHT: f64 = 700.0;

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
            commands::copy_diagnostics,
            update::prepare_update,
            update::apply_update,
            update::cancel_update,
            update::quit_for_update
        ])
        .setup(|app| {
            fit_to_work_area(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start RuxCmd Desktop");
}

/// A fixed 960x700 window overflows on a small or high-DPI monitor (for
/// example 1920x1080 at 150% leaves about 1280x688 of logical space). Because
/// the window cannot be resized by the user, shrink it to the work area here
/// so the window is never taller or wider than the screen.
fn fit_to_work_area(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };

    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let available_width = work.size.width as f64 / scale;
    let available_height = work.size.height as f64 / scale;

    if available_width >= DESIGN_WIDTH && available_height >= DESIGN_HEIGHT {
        return;
    }

    // Leave a small margin so the title bar and window frame stay on screen.
    let width = DESIGN_WIDTH.min(available_width - 16.0).max(640.0);
    let height = DESIGN_HEIGHT.min(available_height - 48.0).max(480.0);
    let _ = window.set_min_size(Some(tauri::LogicalSize::new(width, height)));
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
    let _ = window.center();
}
